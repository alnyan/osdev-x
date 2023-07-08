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
