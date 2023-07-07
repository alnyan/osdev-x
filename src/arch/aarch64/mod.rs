use aarch64_cpu::registers::TTBR1_EL1;
use plat_qemu::PLATFORM;
use tables::KernelTables;

use crate::{
    absolute_address, debug,
    device::{Architecture, Platform},
    mem::ConvertAddress,
};

use self::table::KERNEL_TABLES;

pub mod intrinsics;

pub mod plat_qemu;

pub mod boot;
pub mod exception;
pub mod table;

pub struct AArch64;

#[link_section = ".data.tables"]
pub static mut INITIAL_TABLES: KernelTables = KernelTables::zeroed();

pub static ARCHITECTURE: AArch64 = AArch64;

impl Architecture for AArch64 {
    const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;

    unsafe fn init_mmu(&self) {
        let initial_tables = absolute_address!(INITIAL_TABLES).virtualize() as *const KernelTables;

        KERNEL_TABLES.init(initial_tables);

        TTBR1_EL1.set_baddr(KERNEL_TABLES.l1_physical_address() as u64);
    }
}
