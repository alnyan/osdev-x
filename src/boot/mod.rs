use core::arch::asm;

use aarch64_cpu::registers::{CurrentEL, CPACR_EL1};
use tock_registers::interfaces::{ReadWriteable, Readable};

use crate::mem::KERNEL_VIRT_OFFSET;

const BSP_STACK_SIZE: usize = 32768;

#[repr(C, align(0x20))]
struct KernelStack {
    data: [u8; BSP_STACK_SIZE],
}

static BSP_STACK: KernelStack = KernelStack {
    data: [0; BSP_STACK_SIZE],
};

#[link_section = ".text.entry"]
#[no_mangle]
#[naked]
unsafe extern "C" fn __aarch64_entry() -> ! {
    // Setup the stack and pass on to a proper function
    asm!(
        r#"
        ldr x1, ={stack_bottom} + {stack_size} - {kernel_virt_offset}
        mov sp, x1

        bl {kernel_lower_entry} - {kernel_virt_offset}
"#,
        kernel_lower_entry = sym __aarch64_lower_entry,
        stack_bottom = sym BSP_STACK,
        stack_size = const BSP_STACK_SIZE,
        kernel_virt_offset = const KERNEL_VIRT_OFFSET,
        options(noreturn)
    );
}

extern "C" fn __aarch64_lower_entry(_dtb_phys: usize) -> ! {
    // Unmask FP operations
    CPACR_EL1.modify(CPACR_EL1::FPEN::TrapNothing);

    if CurrentEL.read(CurrentEL::EL) != 1 {
        panic!("Only EL1 is supported for now");
    }

    todo!();
}
