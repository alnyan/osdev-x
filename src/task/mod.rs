//! Multitasking and process/thread management interfaces
use core::sync::atomic::Ordering;

use aarch64_cpu::registers::MPIDR_EL1;
use alloc::{rc::Rc, vec::Vec};
use tock_registers::interfaces::Readable;

use crate::{
    arch::aarch64::{context::TaskContext, cpu::Cpu, smp::CPU_COUNT},
    mem::{
        phys::{self, PageUsage},
        table::{AddressSpace, PageAttributes},
        ConvertAddress,
    },
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

fn func1(_x: usize) {
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

// static USER_CODE: &[u32] = &[0x14000000];

extern "C" fn stats_thread(_x: usize) -> ! {
    let mut counter = 0;
    let this = Process::current();
    let pid = this.id();

    loop {
        for _ in 0..1000000 {
            aarch64_cpu::asm::nop();
        }

        {
            debugln!("+++ STATS +++");
            for (i, queue) in CpuQueue::all().enumerate() {
                let mut lock = queue.lock();
                let total = lock.stats.idle_time + lock.stats.cpu_time;
                if total != 0 {
                    debugln!(
                        "[cpu{}] idle = {}%, cpu = {}%, qsize = {}, current = {}",
                        i,
                        lock.stats.idle_time * 100 / total,
                        lock.stats.cpu_time * 100 / total,
                        lock.queue.len(),
                        lock.current.is_some()
                    );
                } else {
                    debugln!("[cpu{}] N/A", i);
                }

                lock.stats.reset();
            }
            debugln!("--- STATS ---");
        }
        counter += 1;
    }
}

/// Sets up CPU queues and gives them some processes to run
pub fn init() {
    let cpu_count = CPU_COUNT.load(Ordering::Acquire);

    // Create a queue for each CPU
    sched::init_queues(Vec::from_iter((0..cpu_count).map(|_| CpuQueue::new())));

    // Spawn and enqueue some processes
    // for i in 0..12 {
    //     let proc = Process::new_with_context(TaskContext::kernel(__task, i));
    //     proc.enqueue_somewhere();
    // }

    let space = AddressSpace::empty();

    let user_code = 0x100000;
    let user_stack = 0x200000;

    let user_code_phys = phys::alloc_page(PageUsage::Used);

    let user_stack_phys = phys::alloc_page(PageUsage::Used);

    unsafe {
        let write = (user_code_phys as *mut u32).virtualize();

        write.add(0).write_volatile(0xD10023FF);
        write.add(1).write_volatile(0xF90003FF);
        write.add(2).write_volatile(0x14000000);
    }

    space.map_page(user_code, user_code_phys, PageAttributes::AP_BOTH_READWRITE);
    space.map_page(
        user_stack,
        user_stack_phys,
        PageAttributes::AP_BOTH_READWRITE,
    );

    debugln!("space = {:#x}", space.physical_address());
    let proc = Process::new_with_context(TaskContext::user(
        user_code,
        0,
        space.physical_address(),
        user_stack + 0x1000,
    ));

    let q0 = CpuQueue::all().next().unwrap();
    proc.enqueue_to(q0);

    // Spawn kernel stats thread
    let proc = Process::new_with_context(TaskContext::kernel(stats_thread, 0));
    proc.enqueue_to(q0);
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
