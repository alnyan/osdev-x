//! Qemu's "virt" platform implementation for AArch64
use aarch64_cpu::registers::{CNTP_CTL_EL0, CNTP_TVAL_EL0};
use abi::error::Error;
use tock_registers::interfaces::Writeable;

use crate::device::{
    interrupt::{InterruptController, InterruptSource},
    serial::{pl011::Pl011, SerialDevice},
    Device, Platform,
};

use super::{
    gic::{Gic, IrqNumber},
    timer::ArmTimer,
};

/// AArch64 "virt" platform implementation
pub struct QemuPlatform {
    gic: Gic,
    pl011: Pl011,
    local_timer: ArmTimer,
}

impl Platform for QemuPlatform {
    type IrqNumber = IrqNumber;

    const KERNEL_PHYS_BASE: usize = 0x40080000;

    unsafe fn init(&'static self, is_bsp: bool) -> Result<(), Error> {
        if is_bsp {
            self.gic.init()?;

            self.pl011.init_irq()?;

            self.local_timer.init()?;
            self.local_timer.init_irq()?;
        } else {
            self.gic.init_smp_ap()?;

            // TODO somehow merge this with the rest of the code
            CNTP_CTL_EL0.write(CNTP_CTL_EL0::ENABLE::SET + CNTP_CTL_EL0::IMASK::CLEAR);
            CNTP_TVAL_EL0.set(10000000);
            self.gic.enable_irq(IrqNumber::new(30))?;
        }

        Ok(())
    }

    unsafe fn init_primary_serial(&self) {
        self.pl011.init().ok();
    }

    fn name(&self) -> &'static str {
        "qemu"
    }

    fn primary_serial(&self) -> Option<&dyn SerialDevice> {
        Some(&self.pl011)
    }

    fn interrupt_controller(&self) -> &dyn InterruptController<IrqNumber = Self::IrqNumber> {
        &self.gic
    }
}

/// AArch64 "virt" platform
pub static PLATFORM: QemuPlatform = unsafe {
    QemuPlatform {
        pl011: Pl011::new(0x09000000, IrqNumber::new(33)),
        gic: Gic::new(0x08000000, 0x08010000),
        local_timer: ArmTimer::new(IrqNumber::new(30)),
    }
};
