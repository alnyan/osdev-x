//! Per-CPU data structures
use aarch64_cpu::registers::{MPIDR_EL1, TPIDR_EL1};
use alloc::boxed::Box;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{task::sched::CpuQueue, util::OneTimeInit};

/// Per-CPU private data structure
#[repr(C, align(0x10))]
pub struct Cpu {
    id: u32,

    queue: OneTimeInit<&'static CpuQueue>,
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
        (MPIDR_EL1.get() & 0xF) as _
    }
}
