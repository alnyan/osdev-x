use aarch64_cpu::registers::{MPIDR_EL1, TPIDR_EL1};
use alloc::boxed::Box;
use spinning_top::Spinlock;
use tock_registers::interfaces::{Readable, Writeable};

use crate::{sched::CoreScheduler, util::OneTimeInit};

#[repr(C, align(0x10))]
pub struct Cpu {
    id: u32,

    scheduler: OneTimeInit<CoreScheduler>,
}

impl Cpu {
    /// TODO this is still not safe enough
    #[inline(always)]
    pub unsafe fn local<'a>() -> &'a mut Self {
        Self::get_local().unwrap()
    }

    #[inline(always)]
    pub unsafe fn get_local<'a>() -> Option<&'a mut Self> {
        let mut tpidr = TPIDR_EL1.get() as *mut Cpu;
        tpidr.as_mut()
    }

    pub unsafe fn init_local() {
        let this = Box::new(Cpu {
            id: Self::local_id(),
            scheduler: OneTimeInit::new(),
        });
        TPIDR_EL1.set(Box::into_raw(this) as _);
    }

    pub fn init_scheduler(&self, sched: CoreScheduler) {
        self.scheduler.init(sched);
    }

    pub fn scheduler(&self) -> &CoreScheduler {
        self.scheduler.get()
    }

    #[inline(always)]
    pub fn local_id() -> u32 {
        (MPIDR_EL1.get() & 0xF) as _
    }
}
