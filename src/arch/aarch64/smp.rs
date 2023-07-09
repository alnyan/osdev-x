//! Simultaneous multiprocessing support for aarch64
use core::{
    arch::asm,
    sync::atomic::{AtomicUsize, Ordering},
};

use fdt_rs::prelude::PropReader;

use crate::{
    absolute_address,
    arch::aarch64::boot::__aarch64_ap_lower_entry,
    mem::{
        phys::{self, PageUsage},
        ConvertAddress, KERNEL_VIRT_OFFSET,
    },
};

use super::devtree::{self, DeviceTree};

/// ARM Power State Coordination Interface
pub struct Psci {}

/// Number of online CPUs, initially set to 1 (BSP processor is up)
pub static CPU_COUNT: AtomicUsize = AtomicUsize::new(1);

impl Psci {
    /// Function ID for CPU startup request
    const CPU_ON: u32 = 0xC4000003;

    /// Constructs an interface instance for PSCI
    pub const fn new() -> Self {
        Self {}
    }

    #[inline]
    unsafe fn call(&self, mut x0: u64, x1: u64, x2: u64, x3: u64) -> u64 {
        asm!("hvc #0", inout("x0") x0, in("x1") x1, in("x2") x2, in("x3") x3);
        x0
    }

    /// Enables a single processor through a hvc/svc call.
    ///
    /// # Safety
    ///
    /// Calling this outside of initialization sequence or more than once may lead to unexpected
    /// behavior.
    pub unsafe fn cpu_on(&self, target_cpu: usize, entry_point_address: usize, context_id: usize) {
        self.call(
            Self::CPU_ON as _,
            target_cpu as _,
            entry_point_address as _,
            context_id as _,
        );
    }
}

/// Starts application processors using the method specified in the device tree.
///
/// TODO: currently does not handle systems where APs are already started before entry.
///
/// # Safety
///
/// The caller must ensure the physical memory manager was initialized, virtual memory tables are
/// set up and the function has not been called before.
pub unsafe fn start_ap_cores(dt: &DeviceTree) {
    let cpus = dt.node_by_path("/cpus").unwrap();
    let psci = Psci::new();

    for cpu in cpus.children() {
        let Some(compatible) = devtree::find_prop(&cpu, "compatible") else {
            continue;
        };
        let Ok(compatible) = compatible.str() else {
            continue;
        };
        if !compatible.starts_with("arm,cortex-a") {
            continue;
        }

        let reg = devtree::find_prop(&cpu, "reg").unwrap().u32(0).unwrap();
        if reg == 0 {
            continue;
        }

        debugln!(
            "Will start {}, compatible={:?}, reg={}",
            cpu.name().unwrap(),
            compatible,
            reg
        );

        const AP_STACK_PAGES: usize = 2;
        let stack_pages = phys::alloc_pages_contiguous(AP_STACK_PAGES, PageUsage::Used);
        debugln!(
            "{} stack: {:#x}..{:#x}",
            cpu.name().unwrap(),
            stack_pages,
            stack_pages + AP_STACK_PAGES * 0x1000
        );
        // Wait for the CPU to come up
        let old_count = CPU_COUNT.load(Ordering::Acquire);

        psci.cpu_on(
            reg as usize,
            absolute_address!(__aarch64_ap_entry).physicalize(),
            stack_pages + AP_STACK_PAGES * 0x1000,
        );

        while CPU_COUNT.load(Ordering::Acquire) == old_count {
            aarch64_cpu::asm::wfe();
        }

        debugln!("{} is up", cpu.name().unwrap());
    }
}

#[naked]
unsafe extern "C" fn __aarch64_ap_entry() -> ! {
    asm!(
        r#"
        mov sp, x0
        bl {entry} - {kernel_virt_offset}
        "#,
        entry = sym __aarch64_ap_lower_entry,
        kernel_virt_offset = const KERNEL_VIRT_OFFSET,
        options(noreturn)
    );
}
