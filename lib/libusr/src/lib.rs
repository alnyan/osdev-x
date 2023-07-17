#![no_std]

use syscall::{sys_debug_trace, sys_exit};

pub mod debug;
pub mod env;
pub mod process;

extern "Rust" {
    fn main() -> process::ExitCode;
}

pub fn sleep(ns: u64) {
    unsafe { syscall::sys_nanosleep(ns) }
}

#[no_mangle]
extern "C" fn _start(arg: usize) -> ! {
    unsafe {
        env::init_args(arg);

        let result = main();

        sys_exit(result.into_system_exit_code());
    }
}

#[panic_handler]
fn panic_handler(_pi: &core::panic::PanicInfo) -> ! {
    unsafe {
        sys_debug_trace("TODO: panic handler");
    }
    loop {}
}
