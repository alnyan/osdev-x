.global __aarch64_enter_task
.global __aarch64_switch_task

.section .text

__aarch64_task_enter_kernel:
    mov x0, #(1 << 7) | 4
    msr spsr_el1, x0

    # x0 == argument, x1 == entry point
    ldp x0, x1, [sp, #0]
    msr elr_el1, x1

    add sp, sp, #16
    eret

__aarch64_switch_task:
    sub sp, sp, #16 * 6
    stp x19, x20, [sp, #16 * 0]
    stp x21, x22, [sp, #16 * 1]
    stp x23, x24, [sp, #16 * 2]
    stp x25, x26, [sp, #16 * 3]
    stp x27, x28, [sp, #16 * 4]
    stp x29, x30, [sp, #16 * 5]

    mov x19, sp
    str x19, [x1]
__aarch64_enter_task:
    mov sp, x0

    ldp x19, x20, [sp, #16 * 0]
    ldp x21, x22, [sp, #16 * 1]
    ldp x23, x24, [sp, #16 * 2]
    ldp x25, x26, [sp, #16 * 3]
    ldp x27, x28, [sp, #16 * 4]
    ldp x29, x30, [sp, #16 * 5]
    add sp, sp, #16 * 6

    ret
