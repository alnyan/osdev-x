//! Per-CPU data structures
use core::sync::atomic::Ordering;

use aarch64_cpu::registers::{MPIDR_EL1, TPIDR_EL1};
use alloc::{boxed::Box, collections::VecDeque, vec::Vec};
use tock_registers::interfaces::{Readable, Writeable};

use crate::{arch::CpuMessage, sync::IrqSafeSpinlock, task::sched::CpuQueue, util::OneTimeInit};

use super::smp::CPU_COUNT;

/// Per-CPU private data structure
#[repr(C, align(0x10))]
pub struct Cpu {
    id: u32,

    queue: OneTimeInit<&'static CpuQueue>,
}

struct IpiQueue {
    data: IrqSafeSpinlock<Option<CpuMessage>>,
}

static IPI_QUEUES: OneTimeInit<Vec<IpiQueue>> = OneTimeInit::new();

impl IpiQueue {
    pub const fn new() -> Self {
        Self {
            data: IrqSafeSpinlock::new(None),
        }
    }

    pub fn push(&self, msg: CpuMessage) {
        let mut lock = self.data.lock();

        assert!(lock.is_none());
        lock.replace(msg);
    }

    pub fn pop(&self) -> Option<CpuMessage> {
        let mut lock = self.data.lock();
        lock.take()
    }
}

impl Cpu {
    /// Returns a safe reference to the local CPU's private data structure
    #[inline(always)]
    pub fn local<'a>() -> &'a Self {
        Self::get_local().unwrap()
    }

    /// Returns the local CPU data structure reference, if it was set up
    #[inline(always)]
    pub fn get_local<'a>() -> Option<&'a Self> {
        let tpidr = TPIDR_EL1.get() as *mut Cpu;
        unsafe { tpidr.as_ref() }
    }

    /// Sets up the local CPU's private data structure.
    ///
    /// # Safety
    ///
    /// The function is only meant to be called once during the early init process.
    pub unsafe fn init_local() {
        let this = Box::new(Cpu {
            id: Self::local_id(),
            queue: OneTimeInit::new(),
        });
        TPIDR_EL1.set(Box::into_raw(this) as _);
    }

    /// Sets up the local CPU's execution queue.
    pub fn init_queue(&self, queue: &'static CpuQueue) {
        self.queue.init(queue);
    }

    /// Returns the local CPU's execution queue.
    pub fn queue(&self) -> &'static CpuQueue {
        self.queue.get()
    }

    /// Returns the index of the local CPU
    #[inline(always)]
    pub fn local_id() -> u32 {
        (MPIDR_EL1.get() & 0xFF) as _
    }

    pub fn push_ipi_queue(cpu_id: u32, msg: CpuMessage) {
        let ipi_queue = &IPI_QUEUES.get()[cpu_id as usize];
        ipi_queue.push(msg);
    }

    pub fn get_ipi(&self) -> Option<CpuMessage> {
        let ipi_queue = &IPI_QUEUES.get()[self.id as usize];
        ipi_queue.pop()
    }

    pub fn init_ipi_queues() {
        IPI_QUEUES.init(Vec::from_iter(
            (0..CPU_COUNT.load(Ordering::Acquire)).map(|_| IpiQueue::new()),
        ));
    }
}
