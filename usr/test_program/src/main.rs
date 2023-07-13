#![feature(optimize_attribute)]
#![no_std]
#![no_main]

fn print_string(s: &str) -> usize {
    let mut x: usize = 1;
    unsafe {
        core::arch::asm!("svc #0", inout("x0") x, in("x1") s.as_ptr(), in("x2") s.len());
    }
    x
}

static A_STRINGS: &[&str] = &["A", "B", "C", "D"];
static B_STRINGS: &[&str] = &["0", "1", "2", "3"];

#[no_mangle]
extern "C" fn _start(arg: usize) -> ! {
    let mut counter = 0;
    loop {
        let strings = if arg == 0 { A_STRINGS } else { B_STRINGS };
        let string = strings[counter % strings.len()];
        counter += 1;
        print_string(string);

        for _ in 0..1000000 {
            unsafe {
                core::arch::asm!("nop");
            }
        }
    }
}

#[panic_handler]
fn panic_handler(_pi: &core::panic::PanicInfo) -> ! {
    loop {}
}
