//! osdev-x kernel crate
#![feature(
    naked_functions,
    asm_const,
    panic_info_message,
    optimize_attribute,
    const_trait_impl,
    maybe_uninit_slice
)]
#![warn(missing_docs)]
#![no_std]
#![no_main]

extern crate alloc;

use core::sync::atomic::{AtomicBool, Ordering};

use crate::debug::{debug_internal, LogLevel};

#[macro_use]
pub mod debug;
#[macro_use]
pub mod arch;

pub mod device;
pub mod mem;
pub mod sched;
pub mod util;

fn stack_trace(lr: usize, depth: usize) {}

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    static PANIC_HAPPENED: AtomicBool = AtomicBool::new(false);

    if PANIC_HAPPENED
        .compare_exchange(false, true, Ordering::Release, Ordering::Acquire)
        .is_ok()
    {
        log_print_raw!(LogLevel::Fatal, "--- BEGIN PANIC ---\n");
        log_print_raw!(LogLevel::Fatal, "Kernel panic ");

        if let Some(location) = pi.location() {
            log_print_raw!(
                LogLevel::Fatal,
                "at {}:{}:",
                location.file(),
                location.line()
            );
        } else {
            log_print_raw!(LogLevel::Fatal, ":");
        }

        log_print_raw!(LogLevel::Fatal, "\n");

        if let Some(msg) = pi.message() {
            debug_internal(*msg, LogLevel::Fatal);
            log_print_raw!(LogLevel::Fatal, "\n");
        }
        log_print_raw!(LogLevel::Fatal, "---  END PANIC  ---\n");

        loop {}
    } else {
        loop {}
    }
}
