use core::arch::asm;

use aarch64_cpu::registers::{
    CurrentEL, CPACR_EL1, ID_AA64MMFR0_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1,
};
use tables::KernelTables;
use tock_registers::interfaces::{ReadWriteable, Readable};

use crate::mem::KERNEL_VIRT_OFFSET;

const BSP_STACK_SIZE: usize = 32768;

#[repr(C, align(0x20))]
struct KernelStack {
    data: [u8; BSP_STACK_SIZE],
}

#[link_section = ".data.tables"]
#[no_mangle]
static mut KERNEL_TABLES: KernelTables = KernelTables::zeroed();

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

        adr x1, {kernel_tables}
        bl {kernel_lower_entry} - {kernel_virt_offset}
"#,
        kernel_lower_entry = sym __aarch64_lower_entry,
        kernel_tables = sym KERNEL_TABLES,
        stack_bottom = sym BSP_STACK,
        stack_size = const BSP_STACK_SIZE,
        kernel_virt_offset = const KERNEL_VIRT_OFFSET,
        options(noreturn)
    );
}

fn mmu_init(tables_phys: u64) {
    if !ID_AA64MMFR0_EL1.matches_all(ID_AA64MMFR0_EL1::TGran4::Supported) {
        todo!();
    }

    TCR_EL1.modify(
        // General
        TCR_EL1::IPS::Bits_48 +
        // TTBR0
        TCR_EL1::TG0::KiB_4 + TCR_EL1::T0SZ.val(25) + TCR_EL1::SH0::Inner +
        // TTBR1
        TCR_EL1::TG1::KiB_4 + TCR_EL1::T1SZ.val(25) + TCR_EL1::SH1::Outer,
    );

    TTBR0_EL1.set_baddr(tables_phys);
    TTBR1_EL1.set_baddr(tables_phys);

    SCTLR_EL1.modify(SCTLR_EL1::M::Enable);

    loop {}
}

extern "C" fn __aarch64_lower_entry(_dtb_phys: usize, tables_phys: u64) -> ! {
    // Unmask FP operations
    CPACR_EL1.modify(CPACR_EL1::FPEN::TrapNothing);

    if CurrentEL.read(CurrentEL::EL) != 1 {
        panic!("Only EL1 is supported for now");
    }

    mmu_init(tables_phys);

    todo!();
}
