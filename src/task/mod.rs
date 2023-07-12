//! Multitasking and process/thread management interfaces
use core::sync::atomic::Ordering;

use aarch64_cpu::registers::MPIDR_EL1;
use alloc::{rc::Rc, vec::Vec};
use tock_registers::interfaces::Readable;

use crate::{
    arch::aarch64::{context::TaskContext, cpu::Cpu, smp::CPU_COUNT},
    sync::{IrqSafeSpinlock, SpinFence},
    task::sched::CpuQueue,
};

use self::process::Process;

pub mod process;
pub mod sched;

/// Process identifier alias for clarity
pub type ProcessId = usize;

/// Wrapper structure to hold all the system's processes
pub struct ProcessList {
    data: Vec<(ProcessId, Rc<Process>)>,
    last_process_id: ProcessId,
}

impl ProcessList {
    /// Constructs an empty process list
    pub const fn new() -> Self {
        Self {
            last_process_id: 0,
            data: Vec::new(),
        }
    }

    /// Inserts a new process into the list.
    ///
    /// # Safety
    ///
    /// Only meant to be called from inside the Process impl, as this function does not perform any
    /// accounting information updates.
    pub unsafe fn push(&mut self, process: Rc<Process>) -> ProcessId {
        self.last_process_id += 1;
        debugln!("Insert process with ID {}", self.last_process_id);
        self.data.push((self.last_process_id, process));
        self.last_process_id
    }

    /// Looks up a process by its ID
    pub fn get(&self, id: ProcessId) -> Option<&Rc<Process>> {
        self.data
            .iter()
            .find_map(|(i, p)| if *i == id { Some(p) } else { None })
    }
}

/// Global shared process list
pub static PROCESSES: IrqSafeSpinlock<ProcessList> = IrqSafeSpinlock::new(ProcessList::new());

/// Creates a new kernel-space process to execute a closure and queues it to some CPU
pub fn spawn_kernel_closure<F: Fn() + Send + 'static>(f: F) {
    let proc = Process::new_with_context(TaskContext::kernel_closure(f));
    proc.enqueue_somewhere();
}

/// Sets up CPU queues and gives them some processes to run
pub fn init() {
    let cpu_count = CPU_COUNT.load(Ordering::Acquire);

    // Create a queue for each CPU
    sched::init_queues(Vec::from_iter((0..cpu_count).map(|_| CpuQueue::new())));

    // Spawn a closure
    let some_value = 1234;
    spawn_kernel_closure(move || {
        use crate::arch::{Architecture, ArchitectureImpl};

        debugln!("some_value = {}", some_value);

        loop {
            ArchitectureImpl::wait_for_interrupt();
        }
    });
}

/// Sets up the local CPU queue and switches to some task in it for execution.
///
/// # Note
///
/// Any locks held at this point will not be dropped properly, which may lead to a deadlock.
///
/// # Safety
///
/// Only safe to call once at the end of non-threaded system initialization.
pub unsafe fn enter() -> ! {
    static AP_CAN_ENTER: SpinFence = SpinFence::new();

    let cpu_id = MPIDR_EL1.get() & 0xF;

    if cpu_id != 0 {
        // Wait until BSP allows us to enter
        AP_CAN_ENTER.wait_one();
    } else {
        AP_CAN_ENTER.signal();
    }

    let queue = CpuQueue::for_cpu(cpu_id as usize);
    let cpu = Cpu::local();
    cpu.init_queue(queue);

    queue.enter()
}
