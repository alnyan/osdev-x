//! AArch64 Generic Timer

use aarch64_cpu::registers::{CNTP_CTL_EL0, CNTP_TVAL_EL0, MPIDR_EL1};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::{
    arch::PLATFORM,
    device::{interrupt::InterruptSource, Device, Platform},
};

use super::gic::IrqNumber;

pub struct ArmTimer {
    irq: IrqNumber,
}

pub const TICK_INTERVAL: u64 = 10000000;

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
        debugln!("Tick {:#x}", MPIDR_EL1.get());
        CNTP_TVAL_EL0.set(TICK_INTERVAL);
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
    pub const unsafe fn new(irq: IrqNumber) -> Self {
        Self { irq }
    }
}
