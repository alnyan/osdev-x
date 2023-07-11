//! ARM PL011 driver
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use super::SerialDevice;
use crate::{
    arch::{aarch64::gic::IrqNumber, PLATFORM},
    device::{interrupt::InterruptSource, Device, Platform},
    mem::device::DeviceMemoryIo,
    sync::IrqSafeSpinlock,
    util::OneTimeInit,
};

register_bitfields! {
    u32,
    FR [
        TXFF OFFSET(5) NUMBITS(1) [],
        RXFE OFFSET(4) NUMBITS(1) [],
        BUSY OFFSET(3) NUMBITS(1) [],
    ],
    CR [
        RXE OFFSET(9) NUMBITS(1) [],
        TXE OFFSET(8) NUMBITS(1) [],
        UARTEN OFFSET(0) NUMBITS(1) [],
    ],
    ICR [
        ALL OFFSET(0) NUMBITS(11) [],
    ],
    IMSC [
        RXIM OFFSET(4) NUMBITS(1) [],
    ]
}

register_structs! {
    #[allow(non_snake_case)]
    Regs {
        /// Transmit/receive data register
        (0x00 => DR: ReadWrite<u32>),
        (0x04 => _0),
        (0x18 => FR: ReadOnly<u32, FR::Register>),
        (0x1C => _1),
        (0x2C => LCR_H: ReadWrite<u32>),
        (0x30 => CR: ReadWrite<u32, CR::Register>),
        (0x34 => IFLS: ReadWrite<u32>),
        (0x38 => IMSC: ReadWrite<u32, IMSC::Register>),
        (0x3C => _2),
        (0x44 => ICR: WriteOnly<u32, ICR::Register>),
        (0x48 => @END),
    }
}

struct Pl011Inner {
    regs: DeviceMemoryIo<Regs>,
}

/// PL011 device instance
pub struct Pl011 {
    inner: OneTimeInit<IrqSafeSpinlock<Pl011Inner>>,
    base: usize,
    irq: IrqNumber,
}

impl Pl011Inner {
    fn send_byte(&mut self, b: u8) {
        self.regs.DR.set(b as u32);
    }

    unsafe fn init(&mut self) {
        self.regs.CR.set(0);
        self.regs.ICR.write(ICR::ALL::CLEAR);
        self.regs
            .CR
            .write(CR::UARTEN::SET + CR::TXE::SET + CR::RXE::SET);
    }
}

impl SerialDevice for Pl011 {
    fn send(&self, byte: u8) {
        self.inner.get().lock().send_byte(byte);
    }
}

impl InterruptSource for Pl011 {
    unsafe fn init_irq(&'static self) {
        let intc = PLATFORM.interrupt_controller();

        intc.register_handler(self.irq, self);
        self.inner.get().lock().regs.IMSC.modify(IMSC::RXIM::SET);
        intc.enable_irq(self.irq);
    }

    fn handle_irq(&self) {
        let inner = self.inner.get().lock();
        inner.regs.ICR.write(ICR::ALL::CLEAR);

        let byte = inner.regs.DR.get();
        drop(inner);

        debugln!("Got byte {:#x}", byte);
    }
}

impl Device for Pl011 {
    unsafe fn init(&self) {
        let mut inner = Pl011Inner {
            regs: DeviceMemoryIo::map("pl011 UART", self.base),
        };
        inner.init();

        self.inner.init(IrqSafeSpinlock::new(inner));
    }

    fn name(&self) -> &'static str {
        "pl011"
    }
}

impl Pl011 {
    /// Constructs an instance of the device at `base`.
    ///
    /// # Safety
    ///
    /// The caller must ensure the address is valid.
    pub const unsafe fn new(base: usize, irq: IrqNumber) -> Self {
        Self {
            inner: OneTimeInit::new(),
            base,
            irq,
        }
    }
}
