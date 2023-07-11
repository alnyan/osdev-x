//! AArch64-specific task context implementation
use core::{arch::global_asm, cell::UnsafeCell};

use crate::mem::{
    phys::{self, PageUsage},
    ConvertAddress,
};

struct StackBuilder {
    base: usize,
    sp: usize,
}

#[repr(C, align(0x10))]
struct TaskContextInner {
    // 0x00
    sp: usize,
}

/// AArch64 implementation of a task context
pub struct TaskContext {
    inner: UnsafeCell<TaskContextInner>,
}

unsafe impl Sync for TaskContext {}

impl StackBuilder {
    fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            sp: base + size,
        }
    }

    fn push(&mut self, value: usize) {
        if self.sp == self.base {
            panic!();
        }
        self.sp -= 8;
        unsafe {
            (self.sp as *mut usize).write_volatile(value);
        }
    }

    fn _skip(&mut self, count: usize) {
        self.sp -= count * 8;
        if self.sp < self.base {
            panic!();
        }
    }

    fn build(self) -> usize {
        self.sp
    }

    fn init_common(&mut self, entry: usize) {
        self.push(entry); // x30/lr
        self.push(0); // x29
        self.push(0); // x28
        self.push(0); // x27
        self.push(0); // x26
        self.push(0); // x25
        self.push(0); // x24
        self.push(0); // x23
        self.push(0); // x22
        self.push(0); // x21
        self.push(0); // x20
        self.push(0); // x19
    }
}

impl TaskContext {
    /// Constructs a kernel thread context
    pub fn kernel(entry: extern "C" fn(usize) -> !, arg: usize) -> Self {
        const KERNEL_TASK_PAGES: usize = 4;
        let stack_base = unsafe {
            phys::alloc_pages_contiguous(KERNEL_TASK_PAGES, PageUsage::Used).virtualize()
        };

        let mut stack = StackBuilder::new(stack_base, KERNEL_TASK_PAGES * 0x1000);

        // Entry and argument
        stack.push(entry as _);
        stack.push(arg);

        stack.init_common(__aarch64_task_enter_kernel as _);

        let sp = stack.build();

        // TODO stack is leaked

        Self {
            inner: UnsafeCell::new(TaskContextInner { sp }),
        }
    }

    /// Starts execution of `self` task on local CPU.
    ///
    /// # Safety
    ///
    /// Only meant to be called from the scheduler code.
    pub unsafe fn enter(&self) -> ! {
        __aarch64_enter_task(self.inner.get())
    }

    /// Switches from `from` task to `self` task.
    ///
    /// # Safety
    ///
    /// Only meant to be called from the scheduler code.
    pub unsafe fn switch(&self, from: &Self) {
        __aarch64_switch_task(self.inner.get(), from.inner.get())
    }
}

extern "C" {
    fn __aarch64_enter_task(to: *mut TaskContextInner) -> !;
    fn __aarch64_switch_task(to: *mut TaskContextInner, from: *mut TaskContextInner);
    fn __aarch64_task_enter_kernel();
}

global_asm!(include_str!("context.S"));
