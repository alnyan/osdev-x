//! AArch64 architecture and platforms implementation

use core::sync::atomic::Ordering;

use aarch64_cpu::registers::{DAIF, ID_AA64MMFR0_EL1, SCTLR_EL1, TCR_EL1, TTBR0_EL1, TTBR1_EL1};
use abi::error::Error;
use plat_qemu::PLATFORM;
use tock_registers::interfaces::{ReadWriteable, Readable, Writeable};

use crate::{
    absolute_address,
    arch::{
        aarch64::{boot::CPU_INIT_FENCE, cpu::Cpu, devtree::FdtMemoryRegionIter, smp::CPU_COUNT},
        Architecture,
    },
    debug,
    device::platform::Platform,
    fs::devfs,
    mem::{
        heap,
        phys::{self, reserved::reserve_region, PageUsage, PhysicalMemoryRegion},
        ConvertAddress,
    },
    task,
    util::OneTimeInit,
};

use self::{
    devtree::DeviceTree,
    table::{init_fixed_tables, KERNEL_TABLES},
};

pub mod intrinsics;

pub mod plat_qemu;

pub mod boot;
pub mod context;
pub mod cpu;
pub mod devtree;
pub mod exception;
pub mod gic;
pub mod smp;
pub mod table;
pub mod timer;

pub(self) const BOOT_STACK_SIZE: usize = 65536;

#[derive(Clone, Copy)]
#[repr(C, align(0x20))]
pub(self) struct KernelStack {
    data: [u8; BOOT_STACK_SIZE],
}

/// AArch64 platform interface
pub struct AArch64 {
    dt: OneTimeInit<DeviceTree<'static>>,
}

/// Global platform handle
pub static ARCHITECTURE: AArch64 = AArch64 {
    dt: OneTimeInit::new(),
};

impl Architecture for AArch64 {
    const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;

    unsafe fn init_mmu(&self, bsp: bool) {
        if bsp {
            init_fixed_tables();
        }

        let tables_phys = absolute_address!(KERNEL_TABLES).physicalize() as u64;

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
    }

    fn map_device_pages(&self, phys: usize, count: usize) -> Result<usize, Error> {
        unsafe { KERNEL_TABLES.map_device_pages(phys, count) }
    }

    fn wait_for_interrupt() {
        aarch64_cpu::asm::wfi();
    }

    unsafe fn set_interrupt_mask(mask: bool) {
        if mask {
            DAIF.modify(DAIF::I::SET);
        } else {
            DAIF.modify(DAIF::I::CLEAR);
        }
    }

    fn interrupt_mask() -> bool {
        DAIF.read(DAIF::I) != 0
    }
}

impl AArch64 {
    /// Initializes the architecture's device tree
    ///
    /// # Safety
    ///
    /// Only makes sense to call during the early initialization, once.
    pub unsafe fn init_device_tree(&self, dtb_phys: usize) {
        let dt = DeviceTree::from_addr(dtb_phys.virtualize());
        self.dt.init(dt);
    }

    /// Returns the device tree
    ///
    /// # Panics
    ///
    /// Will panic if the device tree has not yet been initialized
    pub fn device_tree(&self) -> &DeviceTree {
        self.dt.get()
    }

    unsafe fn init_physical_memory(&self, dtb_phys: usize) -> Result<(), Error> {
        let dt = self.device_tree();

        reserve_region(
            "dtb",
            PhysicalMemoryRegion {
                base: dtb_phys,
                size: dt.size(),
            },
        );

        let regions = FdtMemoryRegionIter::new(dt);
        phys::init_from_iter(regions)
    }
}

/// AArch64 kernel main entry point
pub fn kernel_main(dtb_phys: usize) -> ! {
    // NOTE it is critical that the code does not panic until the debug is set up, otherwise no
    // message will be displayed

    // Unmap TTBR0
    TTBR0_EL1.set(0);

    unsafe {
        AArch64::set_interrupt_mask(true);

        ARCHITECTURE.init_device_tree(dtb_phys);
        PLATFORM.init_primary_serial();
    }
    // Setup debugging functions
    debug::init();

    exception::init_exceptions();

    debugln!("Initializing {} platform", PLATFORM.name());
    unsafe {
        ARCHITECTURE
            .init_physical_memory(dtb_phys)
            .expect("Failed to initialize the physical memory manager");

        // Setup heap
        let heap_base = phys::alloc_pages_contiguous(16, PageUsage::Used)
            .expect("Could not allocate a block for heap");
        heap::init_heap(heap_base.virtualize(), 16 * 0x1000);

        Cpu::init_local();

        devfs::init();
        PLATFORM.init(true).unwrap();

        let dt = ARCHITECTURE.dt.get();
        if let Err(e) = smp::start_ap_cores(dt) {
            errorln!(
                "Could not initialize AP CPUs: {:?}. Will continue with one CPU.",
                e
            );
        }

        Cpu::init_ipi_queues();

        CPU_INIT_FENCE.signal();
        CPU_INIT_FENCE.wait_all(CPU_COUNT.load(Ordering::Acquire));

        task::init().expect("Failed to initialize the scheduler");

        // Initialize and enter the scheduler
        task::enter();
    }
}
