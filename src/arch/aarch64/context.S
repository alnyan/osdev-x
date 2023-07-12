.global __aarch64_enter_task
.global __aarch64_switch_task

.section .text

.macro SAVE_TASK_STATE
    sub sp, sp, #{context_size}

    stp x19, x20, [sp, #16 * 0]
    stp x21, x22, [sp, #16 * 1]
    stp x23, x24, [sp, #16 * 2]
    stp x25, x26, [sp, #16 * 3]
    stp x27, x28, [sp, #16 * 4]
    stp x29, x30, [sp, #16 * 5]

    mrs x19, tpidr_el0
    mrs x20, ttbr0_el1
    stp x29, x20, [sp, #16 * 6]
.endm

.macro LOAD_TASK_STATE
    // x19 == tpidr_el0, x20 = ttbr0_el1
    ldp x19, x20, [sp, #16 * 6]
    msr tpidr_el0, x19
    msr ttbr0_el1, x20

    ldp x19, x20, [sp, #16 * 0]
    ldp x21, x22, [sp, #16 * 1]
    ldp x23, x24, [sp, #16 * 2]
    ldp x25, x26, [sp, #16 * 3]
    ldp x27, x28, [sp, #16 * 4]
    ldp x29, x30, [sp, #16 * 5]

    add sp, sp, #{context_size}
.endm

__aarch64_task_enter_kernel:
    # EL1h, IRQs unmasked
    mov x0, #5
    msr spsr_el1, x0

    # x0 == argument, x1 == entry point
    ldp x0, x1, [sp, #0]
    msr elr_el1, x1

    add sp, sp, #16

    eret

__aarch64_task_enter_user:
    // x0 == sp, x1 == ignored
    ldp x0, x1, [sp, #16 * 0]
    msr sp_el0, x0

    # EL0t, IRQs unmasked
    msr spsr_el1, xzr

    // x0 == arg, x1 == entry
    ldp x0, x1, [sp, #16 * 1]
    msr elr_el1, x1
    add sp, sp, #32

    mov x1, xzr
    eret

__aarch64_switch_task:
    SAVE_TASK_STATE
    mov x19, sp
    str x19, [x1]

    ldr x0, [x0]
    mov sp, x0
    LOAD_TASK_STATE

    ret

__aarch64_enter_task:
    ldr x0, [x0]
    mov sp, x0
    LOAD_TASK_STATE

    ret
