//! Exception and interrupt management functions
use core::{arch::global_asm, fmt};

use aarch64_cpu::registers::{ELR_EL1, ESR_EL1, FAR_EL1, TTBR0_EL1, TTBR1_EL1, VBAR_EL1};
use tock_registers::interfaces::{Readable, Writeable};

use crate::{
    arch::{aarch64::cpu::Cpu, CpuMessage, PLATFORM},
    debug::LogLevel,
    device::{interrupt::IrqContext, Platform},
    panic::panic_secondary,
};

/// Struct for register values saved when taking an exception
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

/// Initializes the exception/interrupt vectors. May be called repeatedly (though that makes no
/// sense).
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
    let cpu = Cpu::get_local();

    log_print_raw!(LogLevel::Fatal, "SYNC exception:\n");
    log_print_raw!(LogLevel::Fatal, "FAR: {:#x}\n", FAR_EL1.get());
    log_print_raw!(LogLevel::Fatal, "ELR: {:#x}\n", ELR_EL1.get());
    log_print_raw!(LogLevel::Fatal, "TTBR0_EL1: {:#x}\n", TTBR0_EL1.get());
    log_print_raw!(LogLevel::Fatal, "TTBR1_EL1: {:#x}\n", TTBR1_EL1.get());
    log_print_raw!(LogLevel::Fatal, "Register dump:\n");
    log_print_raw!(LogLevel::Fatal, "{:?}\n", frame);

    if let Some(cpu) = cpu {
        let current = cpu.queue().current_process();

        if let Some(current) = current {
            log_print_raw!(LogLevel::Fatal, "In process {}\n", current.id());
        }
    }

    let esr_el1 = ESR_EL1.get();
    let iss = esr_el1 & 0x1FFFFFF;
    let ec = (esr_el1 >> 26) & 0x3F;
    match ec {
        // Data abort from lower level
        0b100100 => {
            log_print_raw!(LogLevel::Fatal, "Exception kind: Data Abort from EL0\n");
            let dfsc = iss & 0x3F;

            if iss & (1 << 24) != 0 {
                let access_size_str = match (iss >> 22) & 0x3 {
                    0 => "i8",
                    1 => "i16",
                    2 => "i32",
                    3 => "i64",
                    _ => unreachable!(),
                };
                let access_type_str = if iss & (1 << 6) != 0 { "write" } else { "read" };

                log_print_raw!(
                    LogLevel::Fatal,
                    "Invalid {} of a {} to/from {:#x}\n",
                    access_type_str,
                    access_size_str,
                    FAR_EL1.get()
                );
            }

            log_print_raw!(LogLevel::Fatal, "DFSC = {:#x}\n", dfsc);
        }
        // Instruction abort from lower level
        0b100000 => {
            log_print_raw!(
                LogLevel::Fatal,
                "Exception kind: Instruction Abort from EL0\n"
            );
            let ifsc = iss & 0x3F;
            log_print_raw!(LogLevel::Fatal, "IFSC = {:#x}\n", ifsc);
        }

        _ => (),
    }

    panic!("Irrecoverable exception");
}

#[no_mangle]
extern "C" fn __aa64_exc_irq_handler(_frame: *mut ExceptionFrame) {
    unsafe {
        let ic = IrqContext::new();
        PLATFORM.interrupt_controller().handle_pending_irqs(&ic);
    }
}

#[no_mangle]
extern "C" fn __aa64_exc_fiq_handler() {
    todo!();
}

#[no_mangle]
extern "C" fn __aa64_exc_serror_handler() {
    todo!();
}

pub(super) fn ipi_handler(msg: Option<CpuMessage>) {
    if let Some(msg) = msg {
        match msg {
            CpuMessage::Panic => panic_secondary(),
        }
    } else {
        warnln!("Spurious IPI received by cpu{}", Cpu::local_id());
        todo!();
    }
    loop {}
}

global_asm!(include_str!("vectors.S"));
