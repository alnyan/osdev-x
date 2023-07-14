#![no_std]
#![no_main]

use libusr::process::ExitCode;

#[macro_use]
extern crate libusr;

static A_STRINGS: &[&str] = &["A", "B", "C", "D"];
static B_STRINGS: &[&str] = &["0", "1", "2", "3"];

#[no_mangle]
fn main() -> ExitCode {
    let arg = libusr::env::args().next().unwrap();

    for counter in 0..100 * (arg + 1) {
        let strings = if arg == 0 { A_STRINGS } else { B_STRINGS };
        let string = strings[counter % strings.len()];
        debug_trace!("{}: {}", counter, string);

        for _ in 0..1000000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }

    ExitCode::from((arg as i32 + 1) * 10)
}
