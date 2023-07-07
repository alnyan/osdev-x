#![feature(naked_functions, asm_const, panic_info_message, optimize_attribute)]
#![no_std]
#![no_main]

use crate::debug::{debug_internal, LogLevel};

#[macro_use]
pub mod debug;

pub mod arch;
pub mod device;
pub mod mem;
pub mod util;

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
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
}
