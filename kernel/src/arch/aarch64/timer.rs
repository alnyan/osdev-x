//! AArch64 Generic Timer

use core::time::Duration;

use aarch64_cpu::registers::{CNTFRQ_EL0, CNTPCT_EL0, CNTP_CTL_EL0, CNTP_TVAL_EL0};
use abi::error::Error;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::{
    arch::PLATFORM,
    device::{interrupt::InterruptSource, platform::Platform, timer::TimestampSource, Device},
    proc::wait,
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

    unsafe fn init(&self) -> Result<(), Error> {
        CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::SET);
        Ok(())
    }
}

impl TimestampSource for ArmTimer {
    fn timestamp(&self) -> Result<Duration, Error> {
        let count = CNTPCT_EL0.get() * 1_000_000;
        let freq = CNTFRQ_EL0.get();

        Ok(Duration::from_nanos((count / freq) * 1_000))
    }
}

impl InterruptSource for ArmTimer {
    fn handle_irq(&self) -> Result<(), Error> {
        CNTP_TVAL_EL0.set(TICK_INTERVAL);
        wait::tick();

        unsafe {
            Cpu::local().queue().yield_cpu();
        }

        Ok(())
    }

    unsafe fn init_irq(&'static self) -> Result<(), Error> {
        let intc = PLATFORM.interrupt_controller();

        intc.register_handler(self.irq, self)?;
        CNTP_CTL_EL0.modify(CNTP_CTL_EL0::IMASK::CLEAR);
        CNTP_TVAL_EL0.set(TICK_INTERVAL);
        intc.enable_irq(self.irq)?;

        Ok(())
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
