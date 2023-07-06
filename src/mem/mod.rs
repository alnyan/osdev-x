use aarch64_cpu::registers::TTBR1_EL1;
use tables::KernelTables;

use crate::mem::fixed::KERNEL_TABLES;

pub mod device;
pub mod fixed;

pub const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;

#[link_section = ".data.tables"]
pub static mut INITIAL_TABLES: KernelTables = KernelTables::zeroed();

pub unsafe trait Virtualize {
    unsafe fn virtualize(self) -> Self;
}

unsafe impl Virtualize for usize {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        #[cfg(debug_assertions)]
        if self > KERNEL_VIRT_OFFSET {
            todo!();
        }

        self + KERNEL_VIRT_OFFSET
    }
}

unsafe impl<T> Virtualize for *mut T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }
}

unsafe impl<T> Virtualize for *const T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }
}

pub unsafe fn mmu_init() {
    let mut initial_tables: usize = 0;
    core::arch::asm!("ldr {0}, ={1}", out(reg) initial_tables, sym INITIAL_TABLES);
    let initial_tables = initial_tables.virtualize() as *const KernelTables;

    KERNEL_TABLES.init(initial_tables);

    TTBR1_EL1.set_baddr((KERNEL_TABLES.l1_ptr() as usize - KERNEL_VIRT_OFFSET) as u64);
}
