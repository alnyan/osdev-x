use core::sync::atomic::AtomicBool;

use aarch64_cpu::registers::MPIDR_EL1;
use alloc::{collections::VecDeque, vec::Vec};
use tock_registers::interfaces::Readable;

use crate::{
    arch::aarch64::{context::TaskContext, cpu::Cpu},
    util::IrqSafeSpinlock,
};

pub type ProcessId = usize;

struct CoreSchedulerInner {
    current: Option<ProcessId>,
    queue: VecDeque<ProcessId>,
}

pub struct CoreScheduler {
    inner: IrqSafeSpinlock<CoreSchedulerInner>,
}

static PROCESSES: IrqSafeSpinlock<Vec<TaskContext>> = IrqSafeSpinlock::new(Vec::new());

impl CoreScheduler {
    pub fn new() -> Self {
        Self {
            inner: IrqSafeSpinlock::new(CoreSchedulerInner {
                current: None,
                queue: VecDeque::new(),
            }),
        }
    }

    pub unsafe fn enter(&self) -> ! {
        let mut processes = PROCESSES.lock();
        let mut inner = self.inner.lock();
        assert!(inner.current.is_none());

        let pid = inner.queue.pop_front().unwrap();
        inner.current = Some(pid);

        drop(inner);
        debugln!("Enter {:#x}", processes[pid].sp);
        let process = &mut processes[pid] as _;

        // TODO this is incorrect
        drop(processes);

        TaskContext::enter(process)
    }

    pub unsafe fn yield_cpu(&self) {
        // TODO handle idle cases
        let mut processes = PROCESSES.lock();
        let mut inner = self.inner.lock();
        let current = inner.current.unwrap();
        inner.queue.push_back(current);

        let next = inner.queue.pop_front().unwrap();
        inner.current = Some(next);

        drop(inner);

        // inner will not be modified (from this core)
        // debugln!(
        //     "From ({}) {:#x} to ({}) {:#x}",
        //     current,
        //     processes[current].sp,
        //     next,
        //     processes[next].sp
        // );
        let from = &mut processes[current] as _;
        let to = &mut processes[next] as _;

        drop(processes);

        TaskContext::switch(to, from)
    }

    pub fn enqueue(&self, id: ProcessId) {
        self.inner.lock().queue.push_back(id)
    }

    pub fn dequeue(&self, id: ProcessId) {
        todo!();
    }
}

fn func1(x: usize) {
    for _ in 0..100000 + x * 10000 {
        aarch64_cpu::asm::nop();
    }
}

extern "C" fn __task(x: usize) -> ! {
    loop {
        debugln!("__task {}", x);
        func1(x);
    }
}

pub unsafe fn enter() -> ! {
    static MAIN_INITIALIZED: AtomicBool = AtomicBool::new(false);

    let core_id = MPIDR_EL1.get() & 0xF;
    let sched = CoreScheduler::new();

    if core_id == 0 {
        let p0 = TaskContext::kernel(__task as _, 1);
        let p1 = TaskContext::kernel(__task as _, 2);

        {
            let mut processes = PROCESSES.lock();
            processes.push(p0);
            processes.push(p1);

            for i in 0..3 {
                processes.push(TaskContext::kernel(__task as _, i * 2 + 3));
                processes.push(TaskContext::kernel(__task as _, i * 2 + 4));
            }
        }

        sched.enqueue(0);
        sched.enqueue(1);
        MAIN_INITIALIZED.store(true, core::sync::atomic::Ordering::Release);
    } else if core_id < 4 {
        while !MAIN_INITIALIZED.load(core::sync::atomic::Ordering::Acquire) {
            aarch64_cpu::asm::nop();
        }

        let id = core_id as usize;
        sched.enqueue(id * 2);
        sched.enqueue(id * 2 + 1);
    } else {
        loop {}
    }

    let cpu = Cpu::local();
    cpu.init_scheduler(sched);

    cpu.scheduler().enter();
}
