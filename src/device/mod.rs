//! Device management and interfaces
use self::{interrupt::InterruptController, serial::SerialDevice};

pub mod interrupt;
pub mod serial;

/// General device interface
pub trait Device {
    /// Initializes the device to a state where it can be used.
    ///
    /// # Safety
    ///
    /// Unsafe to call if the device has already been initialized.
    unsafe fn init(&self);

    /// Returns a display name for the device
    fn name(&self) -> &'static str;
}

/// Platform interface for interacting with a general hardware set
pub trait Platform {
    /// Interrupt number type for the platform
    type IrqNumber;

    /// Address, to which the kernel is expected to be loaded for this platform
    const KERNEL_PHYS_BASE: usize;

    /// Initializes the platform devices to their usable state.
    ///
    /// # Safety
    ///
    /// Unsafe to call if the platform has already been initialized.
    unsafe fn init(&'static self);
    /// Initializes the primary serial device to provide the debugging output as early as possible.
    ///
    /// # Safety
    ///
    /// Unsafe to call if the device has already been initialized.
    unsafe fn init_primary_serial(&self);

    /// Returns a display name for the platform
    fn name(&self) -> &'static str;

    /// Returns a reference to the primary serial device.
    ///
    /// # Note
    ///
    /// May not be initialized at the moment of calling.
    fn primary_serial(&self) -> Option<&dyn SerialDevice>;

    /// Returns a reference to the platform's interrupt controller.
    ///
    /// # Note
    ///
    /// May not be initialized at the moment of calling.
    fn interrupt_controller(&self) -> &dyn InterruptController<IrqNumber = Self::IrqNumber>;
}

/// Interface for an architecture-specific facilities
pub trait Architecture {
    /// Address, to which "zero" address is mapped in the virtual address space
    const KERNEL_VIRT_OFFSET: usize;

    /// Initializes the memory management unit and sets up virtual memory management.
    ///
    /// # Safety
    ///
    /// Unsafe to call if the MMU has already been initialized.
    unsafe fn init_mmu(&self);
}
