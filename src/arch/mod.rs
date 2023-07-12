//! Provides architecture/platform-specific implementation details
pub mod aarch64;

pub use aarch64::plat_qemu::{QemuPlatform as PlatformImpl, PLATFORM};
pub use aarch64::{AArch64 as ArchitectureImpl, ARCHITECTURE};

/// Describes messages sent from some CPU to others
#[derive(Clone, Copy, PartialEq, Debug)]
#[repr(u64)]
pub enum CpuMessage {
    /// Indicates that the sender CPU entered kernel panic and wants other CPUs to follow
    Panic,
}

/// Interface for an architecture-specific facilities
pub trait Architecture {
    /// Address, to which "zero" address is mapped in the virtual address space
    const KERNEL_VIRT_OFFSET: usize;

    /// Initializes the memory management unit and sets up virtual memory management.
    /// `bsp` flag is provided to make sure mapping tables are only initialized once in a SMP
    /// system.
    ///
    /// # Safety
    ///
    /// Unsafe to call if the MMU has already been initialized.
    unsafe fn init_mmu(&self, bsp: bool);

    /// Allocates a virtual mapping for the specified physical memory region
    fn map_device_pages(&self, phys: usize, count: usize) -> usize;

    // Architecture intrinsics

    /// Suspends CPU until an interrupt is received
    fn wait_for_interrupt();

    /// Sets the local CPU's interrupt mask.
    ///
    /// # Safety
    ///
    /// Enabling interrupts may lead to unexpected behavior unless the context explicitly expects
    /// them.
    unsafe fn set_interrupt_mask(mask: bool);

    /// Returns the local CPU's interrupt mask
    fn interrupt_mask() -> bool;
}
