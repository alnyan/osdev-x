use core::{arch::global_asm, fmt};

use aarch64_cpu::registers::{ELR_EL1, FAR_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

#[repr(C)]
pub struct ExceptionFrame {
    r: [u64; 32],
    // ...
}

impl fmt::Debug for ExceptionFrame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for i in (0..32).step_by(2) {
            write!(
                f,
                "x{:<2} = {:#020x}\tx{:<2} = {:#020x}",
                i,
                self.r[i],
                i + 1,
                self.r[i + 1]
            )?;
            if i != 30 {
                f.write_str("\n")?;
            }
        }

        Ok(())
    }
}

pub fn init_exceptions() {
    extern "C" {
        static __aarch64_el1_vectors: u8;
    }
    let vbar = unsafe { &__aarch64_el1_vectors as *const _ };
    VBAR_EL1.set(vbar as u64);
}

#[no_mangle]
extern "C" fn __aa64_exc_sync_handler(frame: *mut ExceptionFrame) {
    let frame = unsafe { &*frame };
    fatalln!("SYNC exception:");
    fatalln!("FAR: {:#x}", FAR_EL1.get());
    fatalln!("ELR: {:#x}", ELR_EL1.get());
    fatalln!("Register dump:");
    fatalln!("{:?}", frame);

    panic!("Irrecoverable exception");
}

#[no_mangle]
extern "C" fn __aa64_exc_irq_handler(_frame: *mut ExceptionFrame) {
    loop {
        aarch64_cpu::asm::nop();
    }
}

#[no_mangle]
extern "C" fn __aa64_exc_fiq_handler() {
    loop {
        aarch64_cpu::asm::nop();
    }
}

#[no_mangle]
extern "C" fn __aa64_exc_serror_handler() {
    loop {
        aarch64_cpu::asm::nop();
    }
}

global_asm!(include_str!("vectors.S"));
