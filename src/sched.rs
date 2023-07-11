use core::sync::atomic::Ordering;

use aarch64_cpu::registers::{CNTPCT_EL0, MPIDR_EL1};
use alloc::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
    vec::Vec,
};
use tock_registers::interfaces::Readable;

use crate::{
    arch::aarch64::{context::TaskContext, cpu::Cpu, smp::CPU_COUNT},
    util::{IrqSafeSpinlock, IrqSafeSpinlockGuard, OneTimeInit, SpinFence},
};

pub type ProcessId = usize;

#[derive(Default)]
pub struct CpuQueueStats {
    idle_time: u64,
    cpu_time: u64,
    measure_time: u64,
}

pub struct CpuQueueInner {
    current: Option<Rc<Process>>,
    queue: VecDeque<Rc<Process>>,

    stats: CpuQueueStats,
}

pub struct CpuQueue {
    inner: IrqSafeSpinlock<CpuQueueInner>,
    idle: OneTimeInit<TaskContext>,
}

pub struct Process {
    context: TaskContext,
}

pub struct ProcessList {
    data: BTreeMap<ProcessId, Rc<Process>>,
    last_process_id: ProcessId,
}

static QUEUES: OneTimeInit<Vec<CpuQueue>> = OneTimeInit::new();
static PROCESSES: IrqSafeSpinlock<ProcessList> = IrqSafeSpinlock::new(ProcessList::new());

impl CpuQueueStats {
    pub fn reset(&mut self) {
        self.cpu_time = 0;
        self.idle_time = 0;
    }
}

impl ProcessList {
    pub const fn new() -> Self {
        Self {
            last_process_id: 0,
            data: BTreeMap::new(),
        }
    }

    pub fn push(&mut self, process: Rc<Process>) {
        self.last_process_id += 1;
        self.data.insert(self.last_process_id, process);
    }

    pub fn get(&self, id: ProcessId) -> Option<&Rc<Process>> {
        self.data.get(&id)
    }
}

impl CpuQueue {
    pub fn new() -> Self {
        Self {
            inner: {
                IrqSafeSpinlock::new(CpuQueueInner {
                    current: None,
                    queue: VecDeque::new(),
                    stats: CpuQueueStats::default(),
                })
            },
            idle: OneTimeInit::new(),
        }
    }

    pub unsafe fn enter(&self) -> ! {
        // First enter the idle thread, because we cannot clone Rc here
        let idle = self.idle.get();

        let t = CNTPCT_EL0.get();
        self.lock().stats.measure_time = t;

        idle.enter()
    }

    pub unsafe fn yield_cpu(&self) {
        let mut inner = self.inner.lock();

        let t = CNTPCT_EL0.get();
        let delta = t - inner.stats.measure_time;
        inner.stats.measure_time = t;

        let current = inner.current.clone();

        if let Some(current) = current.as_ref() {
            inner.queue.push_back(current.clone());

            inner.stats.cpu_time += delta;
        } else {
            inner.stats.idle_time += delta;
        }

        let next = inner.queue.pop_front();

        inner.current = next.clone();

        // Can drop the lock, we hold current and next Rc's
        drop(inner);

        let (from, from_rc) = if let Some(current) = current.as_ref() {
            (&current.context, Rc::strong_count(&current))
        } else {
            (self.idle.get(), 0)
        };

        let (to, to_rc) = if let Some(next) = next.as_ref() {
            (&next.context, Rc::strong_count(&next))
        } else {
            (self.idle.get(), 0)
        };

        to.switch(from)
    }

    pub fn enqueue(&self, p: Rc<Process>) {
        self.inner.lock().queue.push_back(p);
    }

    pub fn dequeue(&self, p: Rc<Process>) {
        todo!();
    }

    pub fn len(&self) -> usize {
        self.inner.lock().queue.len()
    }

    pub fn lock(&self) -> IrqSafeSpinlockGuard<CpuQueueInner> {
        self.inner.lock()
    }
}

fn func1(x: usize) {
    for _ in 0..100000 {
        aarch64_cpu::asm::nop();
    }
}

extern "C" fn __task(x: usize) -> ! {
    loop {
        debugln!("__task {}", x);
        func1(x);
    }
}

extern "C" fn stats_thread() -> ! {
    loop {
        for _ in 0..1000000 {
            aarch64_cpu::asm::nop();
        }
        {
            debugln!("+++ STATS +++");
            for (i, queue) in QUEUES.get().iter().enumerate() {
                let mut lock = queue.lock();
                let total = lock.stats.idle_time + lock.stats.cpu_time;
                if total != 0 {
                    debugln!(
                        "[cpu{}] idle = {}%, cpu = {}%",
                        i,
                        lock.stats.idle_time * 100 / total,
                        lock.stats.cpu_time * 100 / total
                    );
                } else {
                    debugln!("[cpu{}] N/A", i);
                }

                lock.stats.reset();
            }
            debugln!("--- STATS ---");
        }
    }
}

#[naked]
unsafe extern "C" fn __idle() -> ! {
    core::arch::asm!("1: nop; b 1b", options(noreturn));
}

pub fn enqueue(p: Rc<Process>) {
    let queues = QUEUES.get();
    let min_queue = queues.iter().min_by_key(|q| q.len()).unwrap();

    min_queue.enqueue(p)
}

pub unsafe fn init() {
    let cpu_count = CPU_COUNT.load(Ordering::Acquire);
    let mut processes = PROCESSES.lock();

    QUEUES.init(Vec::from_iter((0..cpu_count).map(|i| {
        let queue = CpuQueue::new();
        queue.idle.init(TaskContext::kernel(__idle as _, 0));
        queue
    })));

    // Spawn and enqueue some processes
    for i in 0..2 {
        // processes.push(TaskContext::kernel(__task as _, i));
        let proc = Rc::new(Process {
            context: TaskContext::kernel(__task as _, i),
        });
        processes.push(proc.clone());

        enqueue(proc);
    }

    // Spawn kernel stats thread
    let proc = Rc::new(Process {
        context: TaskContext::kernel(stats_thread as _, 0),
    });
    processes.push(proc.clone());

    enqueue(proc);
}

pub unsafe fn enter() -> ! {
    static AP_CAN_ENTER: SpinFence = SpinFence::new();

    let cpu_id = MPIDR_EL1.get() & 0xF;

    if cpu_id != 0 {
        // Wait until BSP allows us to enter
        AP_CAN_ENTER.wait_one();
    } else {
        AP_CAN_ENTER.signal();
    }

    let queue = &QUEUES.get()[cpu_id as usize];

    let cpu = Cpu::local();
    cpu.init_scheduler(queue);

    queue.enter()
}
