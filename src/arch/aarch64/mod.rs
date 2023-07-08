//! AArch64 architecture and platforms implementation
use aarch64_cpu::registers::TTBR1_EL1;
use plat_qemu::PLATFORM;
use tables::KernelTables;

use crate::{
    absolute_address, debug,
    device::{Architecture, Platform},
    mem::{
        phys::{self, PhysicalMemoryRegion},
        ConvertAddress,
    },
};

use self::table::KERNEL_TABLES;

pub mod intrinsics;

pub mod plat_qemu;

pub mod boot;
pub mod exception;
pub mod table;

/// AArch64 platform interface
pub struct AArch64;

/// Contains compile-time tables for initial kernel setup
#[link_section = ".data.tables"]
pub static mut INITIAL_TABLES: KernelTables = KernelTables::zeroed();

/// Global platform handle
pub static ARCHITECTURE: AArch64 = AArch64;

impl Architecture for AArch64 {
    const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;

    unsafe fn init_mmu(&self) {
        let initial_tables = absolute_address!(INITIAL_TABLES).virtualize() as *const KernelTables;

        KERNEL_TABLES.init(initial_tables);

        TTBR1_EL1.set_baddr(KERNEL_TABLES.l1_physical_address() as u64);
    }
}

/// AArch64 kernel main entry point
pub fn kernel_main(dtb_phys: usize) -> ! {
    // Setup proper debugging functions
    // NOTE it is critical that the code does not panic
    unsafe {
        ARCHITECTURE.init_mmu();
        PLATFORM.init_primary_serial();
    }
    debug::init();
    debugln!("DTB is at {:#20x}", dtb_phys);

    exception::init_exceptions();
    let (page_array_base, page_array_size) = unsafe { KERNEL_TABLES.page_array_range() };

    unsafe {
        phys::init_with_array(
            core::iter::once(PhysicalMemoryRegion {
                base: page_array_base + page_array_size,
                // 8MiB of memory
                size: (4 << 21),
            }),
            page_array_base.virtualize(),
            page_array_size,
        );
    }

    todo!()
}
