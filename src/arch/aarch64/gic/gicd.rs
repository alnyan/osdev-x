//! ARM GICv2 Distributor registers
use spinning_top::Spinlock;
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite, WriteOnly},
};

use crate::{device::interrupt::IpiDeliveryTarget, mem::device::DeviceMemoryIo};

use super::IrqNumber;

register_bitfields! {
    u32,
    CTLR [
        Enable OFFSET(0) NUMBITS(1) []
    ],
    TYPER [
        ITLinesNumber OFFSET(0) NUMBITS(5) []
    ],
    ITARGETSR [
        Offset3 OFFSET(24) NUMBITS(8) [],
        Offset2 OFFSET(16) NUMBITS(8) [],
        Offset1 OFFSET(8) NUMBITS(8) [],
        Offset0 OFFSET(0) NUMBITS(8) []
    ],
    SGIR [
        TargetListFilter OFFSET(24) NUMBITS(2) [
            SpecifiedOnly = 0,
            AllExceptLocal = 1,
            LocalOnly = 2,
        ],
        CPUTargetList OFFSET(16) NUMBITS(8) [],
        INTID OFFSET(0) NUMBITS(4) []
    ],
}

register_structs! {
    #[allow(non_snake_case)]
    pub(super) GicdSharedRegs {
        (0x000 => CTLR: ReadWrite<u32, CTLR::Register>),
        (0x004 => TYPER: ReadWrite<u32, TYPER::Register>),
        (0x008 => _0),
        (0x104 => ISENABLER: [ReadWrite<u32>; 31]),
        (0x180 => _1),
        (0x820 => ITARGETSR: [ReadWrite<u32, ITARGETSR::Register>; 248]),
        (0xC00 => _2),
        (0xC08 => ICFGR: [ReadWrite<u32>; 62]),
        (0xD00 => _3),
        (0xF00 => SGIR: WriteOnly<u32, SGIR::Register>),
        (0xF04 => @END),
    }
}

register_structs! {
    #[allow(non_snake_case)]
    pub(super) GicdBankedRegs {
        (0x000 => _0),
        (0x100 => ISENABLER: ReadWrite<u32>),
        (0x104 => _1),
        (0x800 => ITARGETSR: [ReadOnly<u32, ITARGETSR::Register>; 8]),
        (0x820 => _2),
        (0xC00 => ICFGR: [ReadWrite<u32>; 2]),
        (0xC08 => @END),
    }
}

pub(super) struct Gicd {
    shared_regs: Spinlock<DeviceMemoryIo<GicdSharedRegs>>,
    banked_regs: DeviceMemoryIo<GicdBankedRegs>,
}

impl GicdSharedRegs {
    #[inline(always)]
    fn num_irqs(&self) -> usize {
        ((self.TYPER.read(TYPER::ITLinesNumber) as usize) + 1) * 32
    }

    #[inline(always)]
    fn itargets_slice(&self) -> &[ReadWrite<u32, ITARGETSR::Register>] {
        assert!(self.num_irqs() >= 36);
        let itargetsr_max_index = ((self.num_irqs() - 32) >> 2) - 1;
        &self.ITARGETSR[0..itargetsr_max_index]
    }
}

impl Gicd {
    pub const fn new(
        shared_regs: DeviceMemoryIo<GicdSharedRegs>,
        banked_regs: DeviceMemoryIo<GicdBankedRegs>,
    ) -> Self {
        let shared_regs = Spinlock::new(shared_regs);
        Self {
            shared_regs,
            banked_regs,
        }
    }

    pub unsafe fn set_sgir(&self, target: IpiDeliveryTarget, interrupt_id: u64) {
        assert_eq!(interrupt_id & !0xF, 0);
        let value = match target {
            IpiDeliveryTarget::AllExceptLocal => SGIR::TargetListFilter::AllExceptLocal,
            IpiDeliveryTarget::Specified(mask) => {
                // TODO: need to handle self-ipi case, releasing the lock somehow
                todo!();
            }
        } + SGIR::INTID.val(interrupt_id as u32);

        self.shared_regs.lock().SGIR.write(value);
    }

    fn local_gic_target_mask(&self) -> u32 {
        self.banked_regs.ITARGETSR[0].read(ITARGETSR::Offset0)
    }

    fn enable_irq_inner(&self, irq: usize) {
        let reg = irq >> 5;
        let bit = 1u32 << (irq & 0x1F);

        match reg {
            // Private IRQs
            0 => {
                let reg = &self.banked_regs.ISENABLER;

                reg.set(reg.get() | bit);
            }
            // Shared IRQs
            _ => {
                let regs = self.shared_regs.lock();
                let reg = &regs.ISENABLER[reg - 1];

                reg.set(reg.get() | bit);
            }
        }
    }

    pub fn enable_irq(&self, irq: IrqNumber) {
        let irq = irq.get();

        self.enable_irq_inner(irq);
    }

    pub unsafe fn init(&self) {
        let mask = self.local_gic_target_mask();
        let regs = self.shared_regs.lock();

        debugln!("Enabling GICv2 GICD, max IRQ number: {}", regs.num_irqs());

        regs.CTLR.modify(CTLR::Enable::SET);

        for reg in regs.itargets_slice().iter() {
            // Redirect all IRQs to cpu0 (this CPU)
            reg.write(
                ITARGETSR::Offset0.val(mask)
                    + ITARGETSR::Offset1.val(mask)
                    + ITARGETSR::Offset2.val(mask)
                    + ITARGETSR::Offset3.val(mask),
            );
        }
    }
}
