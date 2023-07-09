//! Main entry point for the AArch64 platforms
use core::arch::asm;

use aarch64_cpu::registers::{CurrentEL, CPACR_EL1, ELR_EL1, SPSR_EL1, SP_EL0};
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use super::{kernel_main, ARCHITECTURE};
use crate::{
    absolute_address,
    device::Architecture,
    mem::{ConvertAddress, KERNEL_VIRT_OFFSET},
};

const BSP_STACK_SIZE: usize = 32768;

#[repr(C, align(0x20))]
struct KernelStack {
    data: [u8; BSP_STACK_SIZE],
}

extern "C" fn __aarch64_lower_entry(dtb_phys: usize) -> ! {
    // Unmask FP operations
    CPACR_EL1.modify(CPACR_EL1::FPEN::TrapNothing);

    if CurrentEL.read(CurrentEL::EL) != 1 {
        panic!("Only EL1 is supported for now");
    }

    unsafe {
        ARCHITECTURE.init_mmu();
    }

    let sp = unsafe { BSP_STACK.data.as_ptr().add(BSP_STACK_SIZE).virtualize() };
    let elr = absolute_address!(__aarch64_upper_entry);
    SP_EL0.set(sp as u64);
    ELR_EL1.set(elr as u64);
    SPSR_EL1.write(SPSR_EL1::M::EL1t);

    unsafe {
        asm!("mov x0, {0}; eret", in(reg) dtb_phys, options(noreturn));
    }
}

extern "C" fn __aarch64_upper_entry(dtb_phys: usize) -> ! {
    kernel_main(dtb_phys);
}

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

#[link_section = ".bss"]
static BSP_STACK: KernelStack = KernelStack {
    data: [0; BSP_STACK_SIZE],
};
