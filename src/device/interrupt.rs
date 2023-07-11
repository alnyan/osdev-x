//! Interrupt-related interfaces
use core::marker::PhantomData;

use crate::arch::CpuMessage;

use super::Device;

/// Specifies the target(s) of interprocessor interrupt delivery
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum IpiDeliveryTarget {
    /// IPI will be delivered to every CPU except the local one
    AllExceptLocal,
    /// IPI will only be sent to CPUs specified in the mask
    Specified(u64),
}

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

    /// Sends a message to the requested set of CPUs through an interprocessor interrupt.
    ///
    /// # Note
    ///
    /// u64 limits the number of targetable CPUs to (only) 64. Platform-specific implementations
    /// may impose narrower restrictions.
    ///
    /// # Safety
    ///
    /// As the call may alter the flow of execution on CPUs, this function is unsafe.
    unsafe fn send_ipi(&self, target: IpiDeliveryTarget, msg: CpuMessage);
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
