//! Per-CPU queue implementation

use aarch64_cpu::registers::CNTPCT_EL0;
use alloc::{collections::VecDeque, rc::Rc, vec::Vec};
use tock_registers::interfaces::Readable;

use crate::{
    arch::aarch64::context::TaskContext,
    sync::{IrqSafeSpinlock, IrqSafeSpinlockGuard},
    util::OneTimeInit,
};

use super::{
    process::{Process, ProcessState},
    ProcessId,
};

/// Per-CPU statistics
#[derive(Default)]
pub struct CpuQueueStats {
    /// Ticks spent idling
    pub idle_time: u64,
    /// Ticks spent running CPU tasks
    pub cpu_time: u64,

    /// Time since last measurement
    measure_time: u64,
}

/// Per-CPU queue's inner data, normally resides under a lock
pub struct CpuQueueInner {
    /// Current process, None if idling
    pub current: Option<Rc<Process>>,
    /// LIFO queue for processes waiting for execution
    pub queue: VecDeque<Rc<Process>>,

    /// CPU time usage statistics
    pub stats: CpuQueueStats,
}

/// Per-CPU queue
pub struct CpuQueue {
    inner: IrqSafeSpinlock<CpuQueueInner>,
    idle: TaskContext,
}

static QUEUES: OneTimeInit<Vec<CpuQueue>> = OneTimeInit::new();

#[naked]
extern "C" fn __idle(_x: usize) -> ! {
    unsafe {
        core::arch::asm!("1: nop; b 1b", options(noreturn));
    }
}

impl CpuQueueStats {
    /// Reset the stats to zero values
    pub fn reset(&mut self) {
        self.cpu_time = 0;
        self.idle_time = 0;
    }
}

impl CpuQueueInner {
    /// Picks a next task for execution, skipping (dropping) those that were suspended. May return
    /// None if the queue is empty or no valid task was found, in which case the scheduler should
    /// go idle.
    pub fn next_ready_task(&mut self) -> Option<Rc<Process>> {
        while !self.queue.is_empty() {
            let task = self.queue.pop_front().unwrap();

            match task.state() {
                ProcessState::Ready => {
                    return Some(task);
                }
                // Drop suspended tasks from the queue
                ProcessState::Suspended => (),
                e => panic!("Unexpected process state in CpuQueue: {:?}", e),
            }
        }

        None
    }

    /// Returns an iterator over all the processes in the queue plus the currently running process,
    /// if there is one.
    pub fn iter(&self) -> impl Iterator<Item = &Rc<Process>> {
        Iterator::chain(self.queue.iter(), self.current.iter())
    }
}

impl CpuQueue {
    /// Constructs an empty queue with its own idle task
    pub fn new() -> Self {
        let idle = TaskContext::kernel(__idle, 0);

        Self {
            inner: {
                IrqSafeSpinlock::new(CpuQueueInner {
                    current: None,
                    queue: VecDeque::new(),
                    stats: CpuQueueStats::default(),
                })
            },
            idle,
        }
    }

    /// Starts queue execution on current CPU.
    ///
    /// # Safety
    ///
    /// Only meant to be called from [crate::task::enter()] function.
    pub unsafe fn enter(&self) -> ! {
        // Start from idle thread to avoid having a Rc stuck here without getting dropped
        let t = CNTPCT_EL0.get();
        self.lock().stats.measure_time = t;
        self.idle.enter()
    }

    /// Yields CPU execution to the next task in queue (or idle task if there aren't any).
    ///
    /// # Safety
    ///
    /// The function is only meant to be called from kernel threads (e.g. if they want to yield
    /// CPU execution to wait for something) or interrupt handlers.
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

        let next = inner.next_ready_task();

        inner.current = next.clone();

        // Can drop the lock, we hold current and next Rc's
        drop(inner);

        let (from, _from_rc) = if let Some(current) = current.as_ref() {
            (current.context(), Rc::strong_count(current))
        } else {
            (&self.idle, 0)
        };

        let (to, _to_rc) = if let Some(next) = next.as_ref() {
            (next.context(), Rc::strong_count(next))
        } else {
            (&self.idle, 0)
        };

        to.switch(from)
    }

    /// Pushes the process to the back of the execution queue.
    ///
    /// # Safety
    ///
    /// Only meant to be called from Process impl. The function does not set any process accounting
    /// information, which may lead to invalid states.
    pub unsafe fn enqueue(&self, p: Rc<Process>) {
        self.inner.lock().queue.push_back(p);
    }

    /// Removes process with given PID from the exeuction queue.
    pub fn dequeue(&self, _pid: ProcessId) {
        todo!();
    }

    /// Returns the queue length at this moment.
    ///
    /// # Note
    ///
    /// This value may immediately change.
    pub fn len(&self) -> usize {
        self.inner.lock().queue.len()
    }

    /// Returns `true` if the queue is empty at the moment.
    ///
    /// # Note
    ///
    /// This may immediately change.
    pub fn is_empty(&self) -> bool {
        self.inner.lock().queue.is_empty()
    }

    /// Returns a safe reference to the inner data structure.
    pub fn lock(&self) -> IrqSafeSpinlockGuard<CpuQueueInner> {
        self.inner.lock()
    }

    /// Returns the process currently being executed.
    ///
    /// # Note
    ///
    /// This function should be safe in all kernel thread/interrupt contexts:
    ///
    /// * (in kthread) the code calling this will still remain on the same thread.
    /// * (in irq) the code cannot be interrupted and other CPUs shouldn't change this queue, so it
    ///            will remain valid until the end of the interrupt or until [CpuQueue::yield_cpu]
    ///            is called.
    pub fn current_process(&self) -> Option<Rc<Process>> {
        self.inner.lock().current.clone()
    }

    /// Returns a queue for given CPU index
    pub fn for_cpu(id: usize) -> &'static CpuQueue {
        &QUEUES.get()[id]
    }

    /// Returns an iterator over all queues of the system
    #[inline]
    pub fn all() -> impl Iterator<Item = &'static CpuQueue> {
        QUEUES.get().iter()
    }

    /// Picks a queue with the least amount of tasks in it
    pub fn least_loaded() -> Option<(usize, &'static CpuQueue)> {
        let queues = QUEUES.get();

        queues.iter().enumerate().min_by_key(|(_, q)| q.len())
    }
}

/// Initializes the global queue list
pub fn init_queues(queues: Vec<CpuQueue>) {
    QUEUES.init(queues);
}
