#![no_std]

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

pub unsafe fn sys_do_something(x: usize, y: usize) -> usize {
    syscall!(SyscallFunction::DoSomething, x, y)
}
