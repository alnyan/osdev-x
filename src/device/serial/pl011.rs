use tock_registers::{interfaces::Writeable, register_structs, registers::ReadWrite};

use super::SerialDevice;
use crate::{device::Device, mem::device::DeviceMemoryIo, util::OneTimeInit};

register_structs! {
    #[allow(non_snake_case)]
    Regs {
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => @END),
    }
}

struct Pl011Inner {
    regs: DeviceMemoryIo<Regs>,
}

pub struct Pl011 {
    inner: OneTimeInit<Pl011Inner>,
    base: usize,
}

impl Pl011Inner {
    fn send_byte(&self, b: u8) {
        self.regs.DR.set(b as u32);
    }
}

impl SerialDevice for Pl011 {
    fn send(&self, byte: u8) {
        self.inner.get().send_byte(byte);
    }
}

impl Device for Pl011 {
    unsafe fn init(&self) {
        self.inner.init(Pl011Inner {
            regs: DeviceMemoryIo::map("pl011", self.base),
        })
    }

    fn name(&self) -> &'static str {
        "pl011"
    }
}

impl Pl011 {
    pub const fn new(base: usize) -> Self {
        Self {
            inner: OneTimeInit::new(),
            base,
        }
    }
}
