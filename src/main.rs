#![feature(naked_functions, asm_const, panic_info_message, optimize_attribute)]
#![no_std]
#![no_main]

use core::sync::atomic::Ordering;

use crate::debug::{EarlyPrint, EarlyPrinter};

#[macro_use]
pub mod debug;

pub mod boot;
pub mod mem;

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    if debug::EARLY_DEBUG_ENABLED.load(Ordering::Acquire) {
        const EARLY_PRINT_ADDR: *mut u8 = 0x9000000 as *mut u8;
        let printer = EarlyPrinter::new(EARLY_PRINT_ADDR);

        if let Some(msg) = pi.message() {
            unsafe {
                printer.early_print("Early kernel panic: ");
            }
            let msg = if let Some(msg) = msg.as_str() {
                msg
            } else {
                "-formatted string-"
            };

            unsafe {
                printer.early_print(msg);
                printer.early_print("\n");
            }
        }

        if let Some(loc) = pi.location() {
            unsafe {
                printer.early_print("In source file: ");
                printer.early_print(loc.file());
                printer.early_print("\n");
            }
        }

        loop {}
    } else {
        loop {}
    }
}
