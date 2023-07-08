//! Utilities for debug information logging
use core::fmt::{self, Arguments};

use crate::{
    arch::PLATFORM,
    device::{serial::SerialDevice, Platform},
    util::{OneTimeInit, SpinLock},
};

/// Defines the severity of the message
#[derive(Clone, Copy)]
pub enum LogLevel {
    /// Debugging and verbose information
    Debug,
    /// General information about transitions in the system state
    Info,
    /// Non-critical abnormalities or notices
    Warning,
    /// Failures of non-essential components
    Error,
    /// Irrecoverable errors which result in kernel panic
    Fatal,
}

struct DebugPrinter {
    sink: &'static dyn SerialDevice,
}

macro_rules! log_print {
    ($level:expr, $args:expr) => {
        $crate::debug::debug_internal($args, $level)
    };
}

macro_rules! debug_tpl {
    ($d:tt $name:ident, $nameln:ident, $level:ident) => {
        #[allow(unused_macros)]
        /// Prints the message to the log
        macro_rules! $name {
            ($d($d args:tt)+) => (log_print!($crate::debug::LogLevel::$level, format_args!($d($d args)+)));
        }

        /// Prints the message to the log, terminated by a newline character
        #[allow(unused_macros)]
        macro_rules! $nameln {
            () => {
                $name!("\n")
            };
            ($d($d args:tt)+) => ($name!("{}\n", format_args!($d($d args)+)));
        }
    };
}

debug_tpl!($ debug, debugln, Debug);
debug_tpl!($ info, infoln, Info);
debug_tpl!($ warn, warnln, Warning);
debug_tpl!($ error, errorln, Error);
debug_tpl!($ fatal, fatalln, Fatal);

static DEBUG_PRINTER: OneTimeInit<SpinLock<DebugPrinter>> = OneTimeInit::new();

impl LogLevel {
    fn log_prefix(self) -> &'static str {
        match self {
            LogLevel::Debug => "",
            LogLevel::Info => "\x1b[37m\x1b[1m",
            LogLevel::Warning => "\x1b[33m\x1b[1m",
            LogLevel::Error => "\x1b[31m\x1b[1m",
            LogLevel::Fatal => "\x1b[38;2;255;0;0m\x1b[1m",
        }
    }

    fn log_suffix(self) -> &'static str {
        match self {
            LogLevel::Debug => "",
            LogLevel::Info => "",
            LogLevel::Warning => "",
            LogLevel::Error => "\x1b[0m",
            LogLevel::Fatal => "\x1b[0m",
        }
    }
}

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            self.sink.send(c);
        }

        Ok(())
    }
}

/// Initializes the debug logging faclities.
///
/// # Panics
///
/// Will panic if called more than once.
pub fn init() {
    DEBUG_PRINTER.init(SpinLock::new(DebugPrinter {
        sink: PLATFORM.primary_serial().unwrap(),
    }));
}

#[doc = "hide"]
pub fn debug_internal(args: Arguments, level: LogLevel) {
    use fmt::Write;

    if DEBUG_PRINTER.is_initialized() {
        let mut printer = DEBUG_PRINTER.get().lock();

        printer.write_str(level.log_prefix()).ok();
        printer.write_fmt(args).ok();
        printer.write_str(level.log_suffix()).ok();
    }
}
