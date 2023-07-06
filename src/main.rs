#![feature(naked_functions, asm_const, panic_info_message, optimize_attribute)]
#![no_std]
#![no_main]

use crate::debug::debug_internal;

#[macro_use]
pub mod debug;

pub mod arch;
pub mod device;
pub mod exception;
pub mod mem;
pub mod util;

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    debugln!("--- BEGIN PANIC ---");
    debug!("Kernel panic ");

    if let Some(location) = pi.location() {
        debugln!("at {}:{}:", location.file(), location.line());
    } else {
        debugln!(":");
    }

    if let Some(msg) = pi.message() {
        debug_internal(*msg);
        debugln!();
    }
    debugln!("---  END PANIC  ---");

    loop {}
}
