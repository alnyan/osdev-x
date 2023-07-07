use crate::device::{
    serial::{pl011::Pl011, SerialDevice},
    Device, Platform,
};

pub struct QemuPlatform {
    pl011: Pl011,
}

impl Platform for QemuPlatform {
    const KERNEL_PHYS_BASE: usize = 0x40080000;

    unsafe fn init(&self) {}

    unsafe fn init_primary_serial(&self) {
        self.pl011.init();
    }

    fn name(&self) -> &'static str {
        "qemu"
    }

    fn primary_serial(&self) -> Option<&dyn SerialDevice> {
        Some(&self.pl011)
    }
}

pub static PLATFORM: QemuPlatform = QemuPlatform {
    pl011: Pl011::new(0x09000000),
};
