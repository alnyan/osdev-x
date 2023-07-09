//! Intrinsic helper functions for AArch64 platforms

use aarch64_cpu::registers::DAIF;
use tock_registers::interfaces::{Readable, Writeable};

/// Returns an absolute address to the given symbol
#[macro_export]
macro_rules! absolute_address {
    ($sym:expr) => {{
        let mut _x: usize;
        unsafe {
            core::arch::asm!("ldr {0}, ={1}", out(reg) _x, sym $sym);
        }
        _x
    }};
}

/// Saves current IRQ state and then masks them
pub fn save_mask_irqs() -> u64 {
    let state = DAIF.get();
    unsafe {
        core::arch::asm!("msr daifset, {bits}", bits = const 2, options(nomem, nostack, preserves_flags));
    }
    state
}

pub unsafe fn restore_irqs(daif: u64) {
    DAIF.set(daif);
}

/// Masks IRQs
pub fn mask_irqs() {
    unsafe {
        core::arch::asm!("msr daifset, 2");
    }
}

/// Unmasks IRQs, allowing their delivery to the CPU.
///
/// # Safety
///
/// The caller must ensure IRQs can actually be handled when calling.
pub unsafe fn unmask_irqs() {
    core::arch::asm!("msr daifclr, 2");
}
