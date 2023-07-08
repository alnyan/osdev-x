//! Intrinsic helper functions for AArch64 platforms

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

/// Unmasks IRQs, allowing their delivery to the CPU.
///
/// # Safety
///
/// The caller must ensure IRQs can actually be handled when calling.
pub unsafe fn unmask_irqs() {
    core::arch::asm!("msr daifclr, 2");
}
