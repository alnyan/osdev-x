//! Main entry point for the AArch64 platforms
use core::{arch::asm, sync::atomic::Ordering};

use aarch64_cpu::registers::{CurrentEL, CPACR_EL1};
use tock_registers::interfaces::{ReadWriteable, Readable};

use super::{
    cpu::Cpu, exception, kernel_main, smp::CPU_COUNT, AArch64, KernelStack, ARCHITECTURE,
    BOOT_STACK_SIZE,
};
use crate::{
    absolute_address,
    arch::{Architecture, PLATFORM},
    device::Platform,
    mem::{ConvertAddress, KERNEL_VIRT_OFFSET},
    sync::SpinFence,
    task,
};

pub(super) static CPU_INIT_FENCE: SpinFence = SpinFence::new();

fn __aarch64_common_lower_entry() {
    // Unmask FP operations
    CPACR_EL1.modify(CPACR_EL1::FPEN::TrapNothing);

    if CurrentEL.read(CurrentEL::EL) != 1 {
        panic!("Only EL1 is supported for now");
    }
}

fn enter_higher_half(sp: usize, elr: usize, arg: usize) -> ! {
    unsafe {
        asm!(r#"
            mov sp, {sp}
            mov x0, {arg}
            br {entry}
            "#, entry = in(reg) elr, arg = in(reg) arg, sp = in(reg) sp, options(noreturn));
    }
}

pub(super) extern "C" fn __aarch64_ap_lower_entry(sp: usize) -> ! {
    __aarch64_common_lower_entry();

    unsafe {
        ARCHITECTURE.init_mmu(false);
    }

    let sp = unsafe { sp.virtualize() };
    let elr = absolute_address!(__aarch64_ap_upper_entry);
    enter_higher_half(sp, elr, 0);
}

extern "C" fn __aarch64_bsp_lower_entry(dtb_phys: usize) -> ! {
    __aarch64_common_lower_entry();

    unsafe {
        ARCHITECTURE.init_mmu(true);
    }

    let sp = unsafe { BSP_STACK.data.as_ptr().add(BOOT_STACK_SIZE).virtualize() };
    let elr = absolute_address!(__aarch64_bsp_upper_entry);
    enter_higher_half(sp as usize, elr, dtb_phys);
}

extern "C" fn __aarch64_bsp_upper_entry(dtb_phys: usize) -> ! {
    kernel_main(dtb_phys);
}

extern "C" fn __aarch64_ap_upper_entry(_x0: usize) -> ! {
    unsafe {
        AArch64::set_interrupt_mask(true);
    }

    // Signal to BSP that we're up
    CPU_COUNT.fetch_add(1, Ordering::SeqCst);
    aarch64_cpu::asm::sev();

    exception::init_exceptions();

    // Initialize CPU-local GIC and timer
    unsafe {
        PLATFORM.init(false).expect("AP platform init failed");

        Cpu::init_local();

        // Synchronize the CPUs to this point
        CPU_INIT_FENCE.signal();
        CPU_INIT_FENCE.wait_all(CPU_COUNT.load(Ordering::Acquire));

        task::enter();
    }
}

#[link_section = ".text.entry"]
#[no_mangle]
#[naked]
unsafe extern "C" fn __aarch64_entry() -> ! {
    // Setup the stack and pass on to a proper function
    asm!(
        r#"
        // Multiple processor cores may or may not be running at this point
        mrs x1, mpidr_el1
        ands x1, x1, #0xF
        bne 1f

        // BSP in SMP or uniprocessor
        ldr x1, ={stack_bottom} + {stack_size} - {kernel_virt_offset}
        mov sp, x1

        bl {kernel_lower_entry} - {kernel_virt_offset}

        // AP in a SMP system
        // TODO spin loop for this method of init
1:
        b .
"#,
        kernel_lower_entry = sym __aarch64_bsp_lower_entry,
        stack_bottom = sym BSP_STACK,
        stack_size = const BOOT_STACK_SIZE,
        kernel_virt_offset = const KERNEL_VIRT_OFFSET,
        options(noreturn)
    );
}

#[link_section = ".bss"]
static BSP_STACK: KernelStack = KernelStack {
    data: [0; BOOT_STACK_SIZE],
};
