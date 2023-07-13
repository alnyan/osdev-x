// vi:ft=a64asm:

.macro EXC_VECTOR el, ht, bits, kind
.p2align 7
    b __aa\bits\()_el\el\ht\()_\kind
.endm

.macro EXC_HANDLER el, ht, bits, kind
__aa\bits\()_el\el\ht\()_\kind:
.if \bits == 32
    // TODO
    b .
.endif

    EXC_SAVE_STATE
    mov x0, sp
    mov lr, xzr
    bl __aa64_exc_\kind\()_handler
    EXC_RESTORE_STATE
    eret
.endm

// 32 gp regs + 3 special regs
.set PT_REGS_SIZE,      (16 * 16 + 16 * 2)

.macro EXC_SAVE_STATE
    sub sp, sp, #PT_REGS_SIZE

    stp x0, x1, [sp, #16 * 0]
    stp x2, x3, [sp, #16 * 1]
    stp x4, x5, [sp, #16 * 2]
    stp x6, x7, [sp, #16 * 3]
    stp x8, x9, [sp, #16 * 4]
    stp x10, x11, [sp, #16 * 5]
    stp x12, x13, [sp, #16 * 6]
    stp x14, x15, [sp, #16 * 7]

    stp x16, x17, [sp, #16 * 8]
    stp x18, x19, [sp, #16 * 9]
    stp x20, x21, [sp, #16 * 10]
    stp x22, x23, [sp, #16 * 11]
    stp x24, x25, [sp, #16 * 12]
    stp x26, x27, [sp, #16 * 13]
    stp x28, x29, [sp, #16 * 14]
    stp x30, x31, [sp, #16 * 15]

    mrs x0, spsr_el1
    mrs x1, elr_el1
    mrs x2, sp_el0

    // TODO
    stp x0, x1, [sp, #16 * 16]
    stp x2, xzr, [sp, #16 * 17]
.endm

.macro EXC_RESTORE_STATE
    ldp x0, x1, [sp, #16 * 16]
    ldp x2, x3, [sp, #16 * 17]

    msr spsr_el1, x0
    msr elr_el1, x1
    msr sp_el0, x2

    ldp x0, x1, [sp, #16 * 0]
    ldp x2, x3, [sp, #16 * 1]
    ldp x4, x5, [sp, #16 * 2]
    ldp x6, x7, [sp, #16 * 3]
    ldp x8, x9, [sp, #16 * 4]
    ldp x10, x11, [sp, #16 * 5]
    ldp x12, x13, [sp, #16 * 6]
    ldp x14, x15, [sp, #16 * 7]

    ldp x16, x17, [sp, #16 * 8]
    ldp x18, x19, [sp, #16 * 9]
    ldp x20, x21, [sp, #16 * 10]
    ldp x22, x23, [sp, #16 * 11]
    ldp x24, x25, [sp, #16 * 12]
    ldp x26, x27, [sp, #16 * 13]
    ldp x28, x29, [sp, #16 * 14]
    ldp x30, x31, [sp, #16 * 15]

    add sp, sp, #PT_REGS_SIZE
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


.p2align 7
EXC_HANDLER 1, t, 64, sync
EXC_HANDLER 1, t, 64, irq
EXC_HANDLER 1, t, 64, fiq
EXC_HANDLER 1, t, 64, serror

EXC_HANDLER 1, h, 64, sync
EXC_HANDLER 1, h, 64, irq
EXC_HANDLER 1, h, 64, fiq
EXC_HANDLER 1, h, 64, serror

EXC_HANDLER 0, t, 64, sync
EXC_HANDLER 0, t, 64, irq
EXC_HANDLER 0, t, 64, fiq
EXC_HANDLER 0, t, 64, serror

EXC_HANDLER 0, t, 32, sync
EXC_HANDLER 0, t, 32, irq
EXC_HANDLER 0, t, 32, fiq
EXC_HANDLER 0, t, 32, serror
