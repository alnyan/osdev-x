use core::fmt::{self, Arguments};

use crate::{
    arch::PLATFORM,
    device::{serial::SerialDevice, Platform},
    util::OneTimeInit,
};

pub struct DebugPrinter {
    sink: &'static dyn SerialDevice,
}

#[allow(unused_macros)]
macro_rules! debug {
    ($($args:tt)+) => (
        $crate::debug::debug_internal(format_args!($($args)+))
    );
}

#[allow(unused_macros)]
macro_rules! debugln {
    () => (debug!("\n"));
    ($($args:tt)+) => (debug!("{}\n", format_args!($($args)+)))
}

static DEBUG_PRINTER: OneTimeInit<DebugPrinter> = OneTimeInit::new();

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.sink.send(c);
        }

        Ok(())
    }
}

pub fn init() {
    DEBUG_PRINTER.init(DebugPrinter {
        sink: PLATFORM.primary_serial().unwrap(),
    });
}

#[doc = "hide"]
pub fn debug_internal(args: Arguments) {
    use fmt::Write;

    if DEBUG_PRINTER.is_initialized() {
        DEBUG_PRINTER.get_mut().write_fmt(args).ok();
    }
}
