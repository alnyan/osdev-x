//! ARM Generic Interrupt Controller v2 driver
use core::sync::atomic::Ordering;

use aarch64_cpu::asm::barrier;
use abi::error::Error;
use spinning_top::Spinlock;

use crate::{
    arch::CpuMessage,
    device::{
        interrupt::{InterruptController, InterruptSource, IpiDeliveryTarget},
        Device,
    },
    mem::device::{DeviceMemory, DeviceMemoryIo},
    util::OneTimeInit,
};

use self::{gicc::Gicc, gicd::Gicd};

use super::{cpu::Cpu, exception::ipi_handler, smp::CPU_COUNT};

const MAX_IRQ: usize = 300;
const IPI_VECTOR: u64 = 1;

pub mod gicc;
pub mod gicd;

/// Wrapper type for ARM interrupt vector
#[repr(transparent)]
#[derive(Clone, Copy)]
pub struct IrqNumber(usize);

/// ARM Generic Interrupt Controller v2
pub struct Gic {
    gicc: OneTimeInit<Gicc>,
    gicd: OneTimeInit<Gicd>,
    gicd_base: usize,
    gicc_base: usize,
    irq_table: Spinlock<[Option<&'static (dyn InterruptSource + Sync)>; MAX_IRQ]>,
}

impl IrqNumber {
    /// Returns the underlying vector number
    #[inline(always)]
    pub const fn get(self) -> usize {
        self.0
    }

    /// Wraps the interrupt vector value in the [IrqNumber] type.
    ///
    /// # Panics
    ///
    /// Will panic if `v` is not a valid interrupt number.
    #[inline(always)]
    pub const fn new(v: usize) -> Self {
        assert!(v < MAX_IRQ);
        Self(v)
    }
}

impl Device for Gic {
    fn name(&self) -> &'static str {
        "ARM Generic Interrupt Controller v2"
    }

    unsafe fn init(&self) -> Result<(), Error> {
        let gicd_mmio = DeviceMemory::map("GICv2 Distributor registers", self.gicd_base, 0x1000)?;
        let gicd_mmio_shared = DeviceMemoryIo::new(gicd_mmio.clone());
        let gicd_mmio_banked = DeviceMemoryIo::new(gicd_mmio);
        let gicc_mmio = DeviceMemoryIo::map("GICv2 CPU registers", self.gicc_base)?;

        let gicd = Gicd::new(gicd_mmio_shared, gicd_mmio_banked);
        let gicc = Gicc::new(gicc_mmio);

        gicd.init();
        gicc.init();

        self.gicd.init(gicd);
        self.gicc.init(gicc);

        Ok(())
    }
}

impl InterruptController for Gic {
    type IrqNumber = IrqNumber;

    fn enable_irq(&self, irq: Self::IrqNumber) -> Result<(), Error> {
        self.gicd.get().enable_irq(irq);
        Ok(())
    }

    fn handle_pending_irqs<'irq>(&'irq self, ic: &crate::device::interrupt::IrqContext<'irq>) {
        let gicc = self.gicc.get();
        let irq_number = gicc.pending_irq_number(ic);
        if irq_number >= MAX_IRQ {
            return;
        }

        gicc.clear_irq(irq_number, ic);

        if irq_number as u64 == IPI_VECTOR {
            // IPI received
            let msg = Cpu::local().get_ipi();
            ipi_handler(msg);
            return;
        }

        {
            let table = self.irq_table.lock();
            match table[irq_number] {
                None => panic!("No IRQ handler registered for irq{}", irq_number),
                Some(handler) => {
                    drop(table);
                    handler.handle_irq().expect("IRQ handler failed");
                }
            }
        }
    }

    fn register_handler(
        &self,
        irq: Self::IrqNumber,
        handler: &'static (dyn InterruptSource + Sync),
    ) -> Result<(), Error> {
        let mut table = self.irq_table.lock();
        let irq = irq.get();
        if table[irq].is_some() {
            return Err(Error::AlreadyExists);
        }

        debugln!("Bound irq{} to {:?}", irq, Device::name(handler));
        table[irq] = Some(handler);

        Ok(())
    }

    unsafe fn send_ipi(&self, target: IpiDeliveryTarget, msg: CpuMessage) -> Result<(), Error> {
        // TODO message queue insertion should be moved
        match target {
            IpiDeliveryTarget::AllExceptLocal => {
                let local = Cpu::local_id();
                for i in 0..CPU_COUNT.load(Ordering::Acquire) {
                    if i != local as usize {
                        Cpu::push_ipi_queue(i as u32, msg);
                    }
                }
            }
            IpiDeliveryTarget::Specified(_) => todo!(),
        }

        // Issue a memory barrier
        barrier::dsb(barrier::ISH);
        barrier::isb(barrier::SY);

        self.gicd.get().set_sgir(target, IPI_VECTOR);

        Ok(())
    }
}

impl Gic {
    /// Constructs an instance of GICv2.
    ///
    /// # Safety
    ///
    /// The caller must ensure the addresses actually point to the GIC components.
    pub const unsafe fn new(gicd_base: usize, gicc_base: usize) -> Self {
        Self {
            gicc: OneTimeInit::new(),
            gicd: OneTimeInit::new(),
            gicd_base,
            gicc_base,
            irq_table: Spinlock::new([None; MAX_IRQ]),
        }
    }

    /// Initializes GICv2 for an application processor.
    ///
    /// # Safety
    ///
    /// Must not be called more than once per each AP. Must not be called from BSP.
    pub unsafe fn init_smp_ap(&self) -> Result<(), Error> {
        self.gicc.get().init();
        Ok(())
    }
}
