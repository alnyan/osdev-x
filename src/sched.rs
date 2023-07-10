use core::{
    arch::global_asm,
    sync::atomic::{AtomicUsize, Ordering},
};

use aarch64_cpu::registers::{ELR_EL1, SPSR_EL1, SP_EL0};
use alloc::vec::Vec;
use spinning_top::Spinlock;
use tock_registers::interfaces::Writeable;

use crate::mem::{
    phys::{self, PageUsage},
    ConvertAddress,
};

pub struct Process {
    sp: usize,
    stack_base: usize,
}

static PROCESSES: Spinlock<Vec<Process>> = Spinlock::new(Vec::new());
static CURRENT_TASK: Spinlock<usize> = Spinlock::new(0);

pub struct ContextStack {
    base: usize,
    sp: usize,
    size: usize,
}

impl ContextStack {
    pub fn new(base: usize, size: usize) -> Self {
        Self {
            base,
            size,
            sp: base + size,
        }
    }

    pub fn push(&mut self, value: usize) {
        if self.sp == self.base {
            panic!();
        }
        self.sp -= 8;
        unsafe {
            (self.sp as *mut usize).write_volatile(value);
        }
    }

    pub fn build(self) -> usize {
        self.sp
    }
}

extern "C" {
    fn __aarch64_enter_task(sp: usize);
    fn __aarch64_switch_task(new_sp: usize, old_sp: *mut usize);
    fn __aarch64_task_enter_kernel();
}

extern "C" fn __task1() -> ! {
    loop {
        debugln!("1");
        for _ in 0..100000 {
            aarch64_cpu::asm::nop();
        }
        unsafe {
            sched_yield();
        }
    }
}

extern "C" fn __task2() -> ! {
    loop {
        debugln!("2");
        for _ in 0..100000 {
            aarch64_cpu::asm::nop();
        }
        unsafe {
            sched_yield();
        }
    }
}

unsafe fn enter_task() -> ! {
    let lock = PROCESSES.lock();
    let process = &lock[0];

    debugln!("Enter with sp {:#x}", process.sp);

    let sp = process.sp;

    // TODO: this is bad and won't be used
    drop(lock);
    __aarch64_enter_task(sp);
    panic!();
}

fn make_process(entry: usize) -> Process {
    const STACK_SIZE: usize = 2;
    let stack_base = unsafe { phys::alloc_pages_contiguous(2, PageUsage::Used).virtualize() };
    let mut stack = ContextStack::new(stack_base, STACK_SIZE * 0x1000);

    // Kernel task data
    stack.push(entry);
    stack.push(0);

    debugln!("entry will be {:#x}", __aarch64_task_enter_kernel as usize);
    // Caller-saved registers
    stack.push(__aarch64_task_enter_kernel as _); // x30/lr
    stack.push(29); // x29
    stack.push(28); // x28
    stack.push(27); // x27
    stack.push(26); // x26
    stack.push(25); // x25
    stack.push(24); // x24
    stack.push(23); // x23
    stack.push(22); // x22
    stack.push(21); // x21
    stack.push(20); // x20
    stack.push(19); // x19

    let sp = stack.build();

    Process { sp, stack_base }
}

pub unsafe fn sched_enter() -> ! {
    // Setup the processes
    PROCESSES.lock().push(make_process(__task1 as _));
    PROCESSES.lock().push(make_process(__task2 as _));

    enter_task();
    loop {}
}

pub unsafe fn sched_yield() {
    let lock = PROCESSES.lock();

    let (curr_idx, next_idx) = {
        let mut idx = CURRENT_TASK.lock();

        let curr_idx = *idx;
        if curr_idx == lock.len() - 1 {
            *idx = 0;
        } else {
            *idx = curr_idx + 1;
        };
        (curr_idx, *idx)
    };

    let prev_process = &lock[curr_idx];
    let next_process = &lock[next_idx];
    let old_sp = (&prev_process.sp) as *const usize as *mut usize;
    let new_sp = next_process.sp;
    debugln!("old_sp = {:#x}, new_sp = {:#x}", prev_process.sp, new_sp);

    drop(lock);
    __aarch64_switch_task(new_sp, old_sp);
}

global_asm!(include_str!("sched.S"));
