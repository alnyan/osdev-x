//! Kernel panic handler code
use core::sync::atomic::{AtomicBool, Ordering};

use crate::{
    arch::{Architecture, ArchitectureImpl, CpuMessage, PLATFORM},
    debug::{debug_internal, LogLevel},
    device::{interrupt::IpiDeliveryTarget, platform::Platform},
    sync::SpinFence,
};

// Just a fence to ensure secondary panics don't trash the screen
static PANIC_FINISHED_FENCE: SpinFence = SpinFence::new();

/// Panic handler for CPUs other than the one that initiated it
pub fn panic_secondary() -> ! {
    unsafe {
        ArchitectureImpl::set_interrupt_mask(true);
    }

    PANIC_FINISHED_FENCE.wait_one();

    log_print_raw!(LogLevel::Fatal, "X");

    loop {
        ArchitectureImpl::wait_for_interrupt();
    }
}

#[panic_handler]
fn panic_handler(pi: &core::panic::PanicInfo) -> ! {
    unsafe {
        ArchitectureImpl::set_interrupt_mask(true);
    }
    static PANIC_HAPPENED: AtomicBool = AtomicBool::new(false);

    if PANIC_HAPPENED
        .compare_exchange(false, true, Ordering::Release, Ordering::Acquire)
        .is_ok()
    {
        // Let other CPUs know we're screwed
        unsafe {
            PLATFORM
                .interrupt_controller()
                .send_ipi(IpiDeliveryTarget::AllExceptLocal, CpuMessage::Panic)
                .ok();
        }

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

        log_print_raw!(LogLevel::Fatal, "X");
        PANIC_FINISHED_FENCE.signal();
    }

    loop {
        ArchitectureImpl::wait_for_interrupt();
    }
}
