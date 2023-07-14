//! Process data structures
use core::sync::atomic::Ordering;

use alloc::rc::Rc;
use atomic_enum::atomic_enum;

use crate::{
    arch::aarch64::{context::TaskContext, cpu::Cpu},
    util::OneTimeInit,
};

use super::{sched::CpuQueue, ProcessId, PROCESSES};

/// Represents the states a process can be at some point in time
#[atomic_enum]
#[derive(PartialEq)]
pub enum ProcessState {
    /// Process is ready for execution and is present in some CPU's queue
    Ready,
    /// Process is currently being executed by some CPU
    Running,
    /// Process is present in a global list, but is not queued for execution until it is resumed
    Suspended,
    /// Process is terminated and waits to be reaped
    Terminated,
}

/// Process data and state structure
pub struct Process {
    context: TaskContext,

    // Process state info
    id: OneTimeInit<ProcessId>,
    state: AtomicProcessState,
}

impl Process {
    /// Creates a process from raw architecture-specific [TaskContext].
    ///
    /// # Note
    ///
    /// Has side-effect of allocating a new PID for itself.
    pub fn new_with_context(context: TaskContext) -> Rc<Self> {
        let this = Rc::new(Self {
            context,
            id: OneTimeInit::new(),
            state: AtomicProcessState::new(ProcessState::Suspended),
        });

        let id = unsafe { PROCESSES.lock().push(this.clone()) };
        this.id.init(id);

        this
    }

    /// Returns a reference to the inner architecture-specific [TaskContext].
    pub fn context(&self) -> &TaskContext {
        &self.context
    }

    /// Returns this process' ID
    pub fn id(&self) -> ProcessId {
        *self.id.get()
    }

    /// Returns the state of the process.
    ///
    /// # Note
    ///
    /// Maybe I should remove this and make ALL state changes atomic.
    pub fn state(&self) -> ProcessState {
        self.state.load(Ordering::Acquire)
    }

    /// Atomically updates the state of the process and returns the previous one.
    pub fn set_state(&self, state: ProcessState) -> ProcessState {
        self.state.swap(state, Ordering::SeqCst)
    }

    /// Selects a suitable CPU queue and submits the process for execution.
    ///
    /// # Panics
    ///
    /// Currently, the code will panic if the process is queued/executing on any queue.
    pub fn enqueue_somewhere(self: Rc<Self>) -> usize {
        // Doesn't have to be precise, so even if something changes, we can still be rebalanced
        // to another CPU
        let (index, queue) = CpuQueue::least_loaded().unwrap();

        self.enqueue_to(queue);

        index
    }

    /// Submits the process to a specific queue.
    ///
    /// # Panics
    ///
    /// Currently, the code will panic if the process is queued/executing on any queue.
    pub fn enqueue_to(self: Rc<Self>, queue: &CpuQueue) {
        let current_state = self.state.swap(ProcessState::Ready, Ordering::SeqCst);

        if current_state != ProcessState::Suspended {
            todo!("Handle attempt to enqueue an already queued/running/terminated process");
        }

        unsafe {
            queue.enqueue(self);
        }
    }

    /// Marks the process as suspended, blocking it from being run until it's resumed.
    ///
    /// # Note
    ///
    /// The process may not halt its execution immediately when this function is called, only when
    /// this function is called targeting the *current process* running on *local* CPU.
    ///
    /// # TODO
    ///
    /// The code currently does not allow suspension of active processes on either local or other
    /// CPUs.
    pub fn suspend(&self) {
        let current_state = self.state.swap(ProcessState::Suspended, Ordering::SeqCst);

        match current_state {
            // NOTE: I'm not sure if the process could've been queued between the store and this
            //       but most likely not (if I'm not that bad with atomics)
            // Do nothing, its queue will just drop the process
            ProcessState::Ready => (),
            // Do nothing, not in a queue already
            ProcessState::Suspended => (),
            ProcessState::Terminated => panic!("Process is terminated"),
            ProcessState::Running => {
                todo!("Cannot dequeue self currently");
            }
        }
    }

    /// Returns the [Process] currently executing on local CPU, None if idling.
    pub fn get_current() -> Option<Rc<Self>> {
        let queue = Cpu::local().queue();
        queue.current_process()
    }

    /// Wraps [Process::get_current()] for cases when the caller is absolutely sure there is a
    /// running process (e.g. the call itself comes from a process).
    pub fn current() -> Rc<Self> {
        Self::get_current().unwrap()
    }

    /// Terminate a process
    pub fn exit(&self, status: usize) {
        let current_state = self.state.swap(ProcessState::Terminated, Ordering::SeqCst);

        debugln!("Process {} exited with code {}", self.id(), status);

        match current_state {
            ProcessState::Suspended => (),
            ProcessState::Ready => todo!(),
            ProcessState::Running => unsafe { Cpu::local().queue().yield_cpu() },
            ProcessState::Terminated => todo!(),
        }
    }
}

impl Drop for Process {
    fn drop(&mut self) {
        infoln!("Drop process!");
    }
}
