use core::fmt;

use syscall::sys_debug_trace;

struct DebugTrace<'a> {
    dst: &'a mut [u8],
    len: usize,
}

impl DebugTrace<'_> {
    fn as_slice(&self) -> &[u8] {
        &self.dst[..self.len]
    }
}

impl fmt::Write for DebugTrace<'_> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.dst[self.len..self.len + s.len()].copy_from_slice(s.as_bytes());
        self.len += s.len();
        Ok(())
    }
}

#[macro_export]
macro_rules! debug_trace {
    ($($args:tt),+) => {
        $crate::debug::debug_trace_internal(format_args!($($args),+))
    };
}

pub fn debug_trace_internal(args: fmt::Arguments) {
    static mut BUFFER: [u8; 1024] = [0; 1024];

    use fmt::Write;
    unsafe {
        let mut trace = DebugTrace {
            dst: &mut BUFFER,
            len: 0,
        };
        trace.write_fmt(args).ok();

        if let Ok(s) = core::str::from_utf8(trace.as_slice()) {
            sys_debug_trace(s);
        }
    }
}
