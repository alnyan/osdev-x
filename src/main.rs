//! osdev-x kernel crate
#![feature(
    naked_functions,
    asm_const,
    panic_info_message,
    optimize_attribute,
    const_trait_impl,
    maybe_uninit_slice
)]
#![allow(clippy::new_without_default)]
#![warn(missing_docs)]
#![no_std]
#![no_main]

use abi::{SyscallArgument, SyscallFunction};

extern crate alloc;

#[macro_use]
pub mod debug;
#[macro_use]
pub mod arch;

pub mod device;
pub mod mem;
pub mod panic;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod util;

/// Entry point for common kernel code.
///
/// # Note
///
/// This function is meant to be used as a kernel-space process after all the platform-specific
/// initialization has finished.
pub fn kernel_main() {
    let mut x0 = SyscallFunction::DoSomething.repr();
    let x1 = 123usize.as_syscall_argument();
    let x2 = 321usize.as_syscall_argument();
    unsafe {
        core::arch::asm!("svc #0", inout("x0") x0, in("x1") x1, in("x2") x2);
    }
    debugln!("Result: {}", x0);
    loop {
        aarch64_cpu::asm::nop();
    }
}
