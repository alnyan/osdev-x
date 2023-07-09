//! Qemu's "virt" platform implementation for AArch64
use crate::{
    debug::LogLevel,
    device::{
        interrupt::{InterruptController, InterruptSource},
        serial::{pl011::Pl011, SerialDevice},
        Device, Platform,
    },
};

use super::{
    devtree::FdtMemoryRegionIter,
    gic::{Gic, IrqNumber},
    ARCHITECTURE,
};

/// AArch64 "virt" platform implementation
pub struct QemuPlatform {
    pl011: Pl011,
    gic: Gic,
}

impl Platform for QemuPlatform {
    type IrqNumber = IrqNumber;

    const KERNEL_PHYS_BASE: usize = 0x40080000;

    unsafe fn init(&'static self) {
        self.gic.init();

        self.pl011.init_irq();
    }

    unsafe fn init_primary_serial(&self) {
        self.pl011.init();
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
    }
};
