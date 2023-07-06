use core::{
    fmt::{self, Arguments},
    sync::atomic::{AtomicBool, Ordering},
};

use crate::{mem::KERNEL_VIRT_OFFSET, util::OneTimeInit};

pub struct EarlyPrinter {
    addr: *mut u8,
}

pub struct DebugPrinter {
    addr: *mut u8,
}

pub trait EarlyPrint<T> {
    unsafe fn early_print(&self, t: T);
}

#[allow(unused_macros)]
macro_rules! debug {
    ($($args:tt)+) => (
        $crate::debug::debug_internal(format_args!($($args)+))
    );
}

#[allow(unused_macros)]
macro_rules! debugln {
    () => {};
    ($($args:tt)+) => (debug!("{}\n", format_args!($($args)+)))
}

pub static EARLY_DEBUG_ENABLED: AtomicBool = AtomicBool::new(true);
pub static DEBUG_PRINTER: OneTimeInit<DebugPrinter> = OneTimeInit::new();

impl EarlyPrinter {
    pub fn new(addr: *mut u8) -> Self {
        Self { addr }
    }
}

impl EarlyPrint<u8> for EarlyPrinter {
    unsafe fn early_print(&self, t: u8) {
        self.addr.write_volatile(t);
    }
}

impl EarlyPrint<&str> for EarlyPrinter {
    unsafe fn early_print(&self, t: &str) {
        let t_len = t.bytes().len();
        let mut t_ptr = t.as_ptr();

        if t_ptr as usize > KERNEL_VIRT_OFFSET {
            t_ptr = (t_ptr as usize - KERNEL_VIRT_OFFSET) as *mut u8;
        }

        for i in 0..t_len {
            self.addr.write_volatile(t_ptr.add(i).read());
        }
    }
}

impl fmt::Write for DebugPrinter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.bytes() {
            unsafe {
                self.addr.write_volatile(c);
            }
        }

        Ok(())
    }
}

pub fn init() {
    DEBUG_PRINTER.init(DebugPrinter {
        addr: 0xFFFFFF8040200000 as *mut u8,
    });
    EARLY_DEBUG_ENABLED.store(false, Ordering::Release);
}

#[doc = "hide"]
pub fn debug_internal(args: Arguments) {
    use fmt::Write;

    if EARLY_DEBUG_ENABLED.load(Ordering::Acquire) {
        loop {}
    } else {
        DEBUG_PRINTER.get_mut().write_fmt(args).ok();
    }
}
