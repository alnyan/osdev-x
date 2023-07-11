//! AArch64 Generic Timer
use core::sync::atomic::AtomicU64;

use aarch64_cpu::registers::{CNTP_CTL_EL0, CNTP_TVAL_EL0, MPIDR_EL1, SP_EL0};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::{
    arch::PLATFORM,
    device::{interrupt::InterruptSource, Device, Platform},
};

use super::{cpu::Cpu, gic::IrqNumber};

/// ARM Generic Timer driver
pub struct ArmTimer {
    irq: IrqNumber,
}

/// ARM timer tick interval (in some time units?)
pub const TICK_INTERVAL: u64 = 1000000;

impl Device for ArmTimer {
    fn name(&self) -> &'static str {
        "ARM Generic Timer"
    }

    unsafe fn init(&self) {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);
    }
}

impl InterruptSource for ArmTimer {
    fn handle_irq(&self) {
        CNTP_TVAL_EL0.set(TICK_INTERVAL);

        unsafe {
            Cpu::local().queue().yield_cpu();
        }
    }

    unsafe fn init_irq(&'static self) {
        let intc = PLATFORM.interrupt_controller();

        intc.register_handler(self.irq, self);
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK::CLEAR);
        CNTP_TVAL_EL0.set(TICK_INTERVAL);
        intc.enable_irq(self.irq);
    }
}

impl ArmTimer {
    /// Constructs an instance of ARM generic timer.
    ///
    /// # Safety
    ///
    /// The caller must ensure the function has not been called before.
    pub const unsafe fn new(irq: IrqNumber) -> Self {
        Self { irq }
    }
}
