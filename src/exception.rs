use core::arch::global_asm;

use aarch64_cpu::registers::VBAR_EL1;
use tock_registers::interfaces::Writeable;

pub fn init_exceptions() {
    extern "C" {
        static __aarch64_el1_vectors: u8;
    }
    let vbar = unsafe { &__aarch64_el1_vectors as *const _ };
    VBAR_EL1.set(vbar as u64);
}

global_asm!(
    r#"
.macro EXC_VECTOR el, ht, bits, kind
.p2align 7
    b .
.endm

.section .text
.p2align 12
__aarch64_el1_vectors:
    EXC_VECTOR 1, t, 64, sync
    EXC_VECTOR 1, t, 64, irq
    EXC_VECTOR 1, t, 64, fiq
    EXC_VECTOR 1, t, 64, serror

    EXC_VECTOR 1, h, 64, sync
    EXC_VECTOR 1, h, 64, irq
    EXC_VECTOR 1, h, 64, fiq
    EXC_VECTOR 1, h, 64, serror

    EXC_VECTOR 0, t, 64, sync
    EXC_VECTOR 0, t, 64, irq
    EXC_VECTOR 0, t, 64, fiq
    EXC_VECTOR 0, t, 64, serror

    EXC_VECTOR 0, t, 32, sync
    EXC_VECTOR 0, t, 32, irq
    EXC_VECTOR 0, t, 32, fiq
    EXC_VECTOR 0, t, 32, serror
"#
);
