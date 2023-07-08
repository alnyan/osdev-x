//! ARM GICv2 Distributor registers
use tock_registers::{
    interfaces::{ReadWriteable, Readable, Writeable},
    register_bitfields, register_structs,
    registers::{ReadOnly, ReadWrite},
};

use crate::{mem::device::DeviceMemoryIo, util::SpinLock};

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
    ]
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
        (0xD00 => @END),
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
    shared_regs: SpinLock<DeviceMemoryIo<GicdSharedRegs>>,
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
        let shared_regs = SpinLock::new(shared_regs);
        Self {
            shared_regs,
            banked_regs,
        }
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
