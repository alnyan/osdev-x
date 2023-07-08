//! Interrupt-related interfaces
use core::marker::PhantomData;

use super::Device;

/// Interface for a device capable of emitting interrupts
pub trait InterruptSource: Device {
    /// Initializes and enables IRQs for the device.
    ///
    /// # Safety
    ///
    /// The caller must ensure the function hasn't been called before.
    unsafe fn init_irq(&'static self);

    /// Handles the interrupt raised by the device
    fn handle_irq(&self);
}

/// Interface for a device responsible for routing and handling IRQs
pub trait InterruptController: Device {
    /// Interrupt number wrapper type
    type IrqNumber;

    /// Binds an interrupt number to its handler implementation
    fn register_handler(
        &self,
        irq: Self::IrqNumber,
        handler: &'static (dyn InterruptSource + Sync),
    );

    /// Enables given interrupt number/vector
    fn enable_irq(&self, irq: Self::IrqNumber);

    /// Handles all pending interrupts on this controller
    fn handle_pending_irqs<'irq>(&'irq self, ic: &IrqContext<'irq>);
}

/// Token type to indicate that the code is being run from an interrupt handler
pub struct IrqContext<'irq> {
    _0: PhantomData<&'irq ()>,
}

impl<'irq> IrqContext<'irq> {
    /// Constructs an IRQ context token
    ///
    /// # Safety
    ///
    /// Only allowed to be constructed in top-level IRQ handlers
    #[inline(always)]
    pub const unsafe fn new() -> Self {
        Self { _0: PhantomData }
    }
}
