//! AArch64 architecture and platforms implementation
use aarch64_cpu::registers::{ID_AA64MMFR0_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1};
use plat_qemu::PLATFORM;
use tock_registers::interfaces::{ReadWriteable, Readable};

use crate::{
    absolute_address,
    arch::aarch64::{devtree::FdtMemoryRegionIter, table::init_fixed_tables},
    debug,
    device::{Architecture, Platform},
    mem::{
        phys::{self, PhysicalMemoryRegion, RESERVED_MEMORY},
        ConvertAddress,
    },
    util::OneTimeInit,
};

use self::{devtree::DeviceTree, table::KERNEL_TABLES};

pub mod intrinsics;

pub mod plat_qemu;

pub mod boot;
pub mod devtree;
pub mod exception;
pub mod gic;
pub mod table;

/// AArch64 platform interface
pub struct AArch64 {
    dt: OneTimeInit<DeviceTree<'static>>,
}

/// Global platform handle
pub static ARCHITECTURE: AArch64 = AArch64 {
    dt: OneTimeInit::new(),
};

impl Architecture for AArch64 {
    const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;

    unsafe fn init_mmu(&self) {
        init_fixed_tables();

        let tables_phys = absolute_address!(KERNEL_TABLES).physicalize() as u64;

        if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran4::Supported) {
            todo!();
        }

        TCR_EL1.modify(
            // General
            TCR_EL1::IPS::Bits_48 +
            // TTBR0
            TCR_EL1::TG0::KiB_4 + TCR_EL1::T0SZ.val(25) + TCR_EL1::SH0::Inner +
            // TTBR1
            TCR_EL1::TG1::KiB_4 + TCR_EL1::T1SZ.val(25) + TCR_EL1::SH1::Outer,
        );

        TTBR0_EL1.set_baddr(tables_phys);
        TTBR1_EL1.set_baddr(tables_phys);

        SCTLR_EL1.modify(SCTLR_EL1::M::Enable);
    }

    fn map_device_pages(&self, phys: usize, count: usize) -> usize {
        unsafe { KERNEL_TABLES.map_device_pages(phys, count) }
    }
}

impl AArch64 {
    pub unsafe fn init_device_tree(&self, dtb_phys: usize) {
        let dt = DeviceTree::from_addr(dtb_phys.virtualize());
        self.dt.init(dt);
    }

    pub fn device_tree(&self) -> &DeviceTree {
        self.dt.get()
    }

    unsafe fn init_physical_memory(&self, dtb_phys: usize) {
        let dt = self.device_tree();

        RESERVED_MEMORY.reserve(
            "dtb",
            PhysicalMemoryRegion {
                base: dtb_phys,
                size: dt.size(),
            },
        );

        let regions = FdtMemoryRegionIter::new(dt);
        phys::init_from_iter(regions);
    }
}

/// AArch64 kernel main entry point
pub fn kernel_main(dtb_phys: usize) -> ! {
    // Setup proper debugging functions
    // NOTE it is critical that the code does not panic
    unsafe {
        ARCHITECTURE.init_device_tree(dtb_phys);
        PLATFORM.init_primary_serial();
    }
    debug::init();

    exception::init_exceptions();

    debugln!("Initializing {} platform", PLATFORM.name());
    unsafe {
        ARCHITECTURE.init_physical_memory(dtb_phys);
        PLATFORM.init();
        intrinsics::unmask_irqs();
    }

    loop {
        aarch64_cpu::asm::wfi();
    }
}
