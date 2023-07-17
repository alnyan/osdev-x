//! Device management and interfaces
use abi::error::Error;

pub mod interrupt;
pub mod platform;
pub mod serial;
pub mod timer;

/// General device interface
pub trait Device {
    /// Initializes the device to a state where it can be used.
    ///
    /// # Safety
    ///
    /// Unsafe to call if the device has already been initialized.
    unsafe fn init(&self) -> Result<(), Error>;

    /// Returns a display name for the device
    fn name(&self) -> &'static str;
}
