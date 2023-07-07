use aarch64_cpu::registers::TTBR1_EL1;
use tables::KernelTables;

use crate::{absolute_address, mem::fixed::KERNEL_TABLES};

use self::table::{EntryLevel, L1, L2, L3};

pub mod device;
pub mod fixed;
pub mod table;

pub const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;
pub const KERNEL_PHYS_BASE: usize = 0x40080000;

#[link_section = ".data.tables"]
pub static mut INITIAL_TABLES: KernelTables = KernelTables::zeroed();

#[const_trait]
pub trait VirtualAddressParts {
    fn l1i(self) -> usize;
    fn l2i(self) -> usize;
    fn l3i(self) -> usize;
}

pub unsafe trait ConvertAddress {
    unsafe fn virtualize(self) -> Self;
    unsafe fn physicalize(self) -> Self;
}

unsafe impl ConvertAddress for usize {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        #[cfg(debug_assertions)]
        if self > KERNEL_VIRT_OFFSET {
            todo!();
        }

        self + KERNEL_VIRT_OFFSET
    }

    #[inline(always)]
    unsafe fn physicalize(self) -> Self {
        #[cfg(debug_assertions)]
        if self < KERNEL_VIRT_OFFSET {
            todo!();
        }

        self - KERNEL_VIRT_OFFSET
    }
}

unsafe impl<T> ConvertAddress for *mut T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }

    #[inline(always)]
    unsafe fn physicalize(self) -> Self {
        (self as usize).physicalize() as Self
    }
}

unsafe impl<T> ConvertAddress for *const T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }

    #[inline(always)]
    unsafe fn physicalize(self) -> Self {
        (self as usize).physicalize() as Self
    }
}

impl const VirtualAddressParts for usize {
    fn l1i(self) -> usize {
        L1::index(self)
    }

    fn l2i(self) -> usize {
        L2::index(self)
    }

    fn l3i(self) -> usize {
        L3::index(self)
    }
}

pub unsafe fn mmu_init() {
    let initial_tables = absolute_address!(INITIAL_TABLES).virtualize() as *const KernelTables;

    KERNEL_TABLES.init(initial_tables);

    TTBR1_EL1.set_baddr(KERNEL_TABLES.l1_physical_address() as u64);
}
