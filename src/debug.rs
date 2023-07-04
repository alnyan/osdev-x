use core::{
    fmt::Arguments,
    sync::atomic::{AtomicBool, Ordering},
};

use crate::mem::KERNEL_VIRT_OFFSET;

pub struct EarlyPrinter {
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

#[doc = "hide"]
pub fn debug_internal(_args: Arguments) {
    if EARLY_DEBUG_ENABLED.load(Ordering::Acquire) {
        loop {}
    } else {
        loop {}
    }
}
