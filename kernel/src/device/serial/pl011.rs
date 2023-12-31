//! ARM PL011 driver
use abi::error::Error;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};
use vfs::CharDevice;

use super::SerialDevice;
use crate::{
    arch::{aarch64::gic::IrqNumber, PLATFORM},
    device::{
        interrupt::InterruptSource,
        platform::Platform,
        tty::{CharRing, TtyDevice},
        Device,
    },
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
    ring: CharRing<16>,
}

impl Pl011Inner {
    fn send_byte(&mut self, b: u8) -> Result<(), Error> {
        while self.regs.FR.matches_all(FR::TXFF::SET) {
            core::hint::spin_loop();
        }
        self.regs.DR.set(b as u32);
        Ok(())
    }

    fn recv_byte(&mut self, blocking: bool) -> Result<u8, Error> {
        if self.regs.FR.matches_all(FR::RXFE::SET) {
            if !blocking {
                todo!();
            }
            while self.regs.FR.matches_all(FR::RXFE::SET) {
                core::hint::spin_loop();
            }
        }

        Ok(self.regs.DR.get() as u8)
    }

    unsafe fn init(&mut self) {
        self.regs.CR.set(0);
        self.regs.ICR.write(ICR::ALL::CLEAR);
        self.regs
            .CR
            .write(CR::UARTEN::SET + CR::TXE::SET + CR::RXE::SET);
    }
}

impl TtyDevice<16> for Pl011 {
    fn ring(&self) -> &CharRing<16> {
        &self.ring
    }
}

impl CharDevice for Pl011 {
    fn write(&self, blocking: bool, data: &[u8]) -> Result<usize, Error> {
        assert!(blocking);
        self.line_write(data)
    }

    fn read(&'static self, blocking: bool, data: &mut [u8]) -> Result<usize, Error> {
        assert!(blocking);
        self.line_read(data)
    }
}

impl SerialDevice for Pl011 {
    fn send(&self, byte: u8) -> Result<(), Error> {
        self.inner.get().lock().send_byte(byte)
    }

    fn receive(&self, blocking: bool) -> Result<u8, Error> {
        self.inner.get().lock().recv_byte(blocking)
    }
}

impl InterruptSource for Pl011 {
    unsafe fn init_irq(&'static self) -> Result<(), Error> {
        let intc = PLATFORM.interrupt_controller();

        intc.register_handler(self.irq, self)?;
        self.inner.get().lock().regs.IMSC.modify(IMSC::RXIM::SET);
        intc.enable_irq(self.irq)?;

        Ok(())
    }

    fn handle_irq(&self) -> Result<(), Error> {
        let inner = self.inner.get().lock();
        inner.regs.ICR.write(ICR::ALL::CLEAR);

        let byte = inner.regs.DR.get();
        drop(inner);

        self.recv_byte(byte as u8);

        Ok(())
    }
}

impl Device for Pl011 {
    unsafe fn init(&self) -> Result<(), Error> {
        let mut inner = Pl011Inner {
            regs: DeviceMemoryIo::map("pl011 UART", self.base)?,
        };
        inner.init();

        self.inner.init(IrqSafeSpinlock::new(inner));
        Ok(())
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
            ring: CharRing::new(),
            base,
            irq,
        }
    }
}
