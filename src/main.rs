//! osdev-x kernel crate
#![feature(
    naked_functions,
    asm_const,
    panic_info_message,
    optimize_attribute,
    const_trait_impl
)]
#![warn(missing_docs)]
#![no_std]
#![no_main]

use core::sync::atomic::{AtomicBool, Ordering};

use crate::debug::{debug_internal, LogLevel};

#[macro_use]
pub mod debug;
#[macro_use]
pub mod arch;

pub mod device;
pub mod mem;
pub mod util;

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    static PANIC_HAPPENED: AtomicBool = AtomicBool::new(false);

    if PANIC_HAPPENED
        .compare_exchange(false, true, Ordering::Release, Ordering::Acquire)
        .is_ok()
    {
        fatalln!("--- BEGIN PANIC ---");
        fatal!("Kernel panic ");

        if let Some(location) = pi.location() {
            fatalln!("at {}:{}:", location.file(), location.line());
        } else {
            fatalln!(":");
        }

        if let Some(msg) = pi.message() {
            debug_internal(*msg, LogLevel::Fatal);
            fatalln!();
        }
        fatalln!("---  END PANIC  ---");

        loop {}
    } else {
        loop {}
    }
}
