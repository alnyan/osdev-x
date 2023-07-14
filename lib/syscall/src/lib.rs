#![no_std]

use abi::SyscallFunction;

macro_rules! syscall {
    ($num:expr) => {{
        let mut res: usize;
        core::arch::asm!("svc #0", out("x0") res, in("x8") $num.repr(), options(nostack));
        res
    }};
    ($num:expr, $a0:expr) => {{
        let mut res: usize = $a0;
        core::arch::asm!("svc #0",
             inout("x0") res,
             in("x8") $num.repr(), options(nostack));
        res
    }};
    ($num:expr, $a0:expr, $a1:expr) => {{
        let mut res: usize = $a0;
        core::arch::asm!("svc #0",
             inout("x0") res, in("x1") $a1,
             in("x8") $num.repr(), options(nostack));
        res
    }};
    ($num:expr, $a0:expr, $a1:expr, $a2:expr) => {{
        let mut res: usize = $a0;
        core::arch::asm!("svc #0",
             inout("x0") res, in("x1") $a1, in("x2") $a2,
             in("x8") $num.repr(), options(nostack));
        res
    }};
    ($num:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr) => {{
        let mut res: usize = $a0;
        core::arch::asm!("svc #0",
             inout("x0") res, in("x1") $a1, in("x2") $a2,
             in("x3") $a3, in("x8") $num.repr(), options(nostack));
        res
    }};
    ($num:expr, $a0:expr, $a1:expr, $a2:expr, $a3:expr, $a4:expr) => {{
        let mut res: usize = $a0;
        core::arch::asm!("svc #0",
             inout("x0") res, in("x1") $a1, in("x2") $a2,
             in("x3") $a3, in("x4") $a4, in("x8") $num.repr(), options(nostack));
        res
    }};
}

macro_rules! argn {
    ($a:expr) => {
        $a as usize
    };
}

macro_rules! argp {
    ($a:expr) => {
        $a as usize
    };
}

/// [SyscallFunction::DebugTrace] call.
///
/// * s: message to print to the system trace.
///
/// # Safety
///
/// Unsafe: direct system call.
pub unsafe fn sys_debug_trace(s: &str) -> usize {
    syscall!(
        SyscallFunction::DebugTrace,
        argp!(s.as_ptr()),
        argn!(s.len())
    )
}

/// [SyscallFunction::Exit] call.
///
/// * code: process termination status code.
///
/// # Safety
///
/// Unsafe: direct system call.
pub unsafe fn sys_exit(code: i32) -> ! {
    syscall!(SyscallFunction::Exit, argn!(code));
    panic!();
}
