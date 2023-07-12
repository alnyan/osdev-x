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

use task::process::Process;

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
    Process::current().exit(0);
    loop {
        debugln!("TEST");
        aarch64_cpu::asm::nop();
    }
}
