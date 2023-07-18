//! Serial device interfaces
use abi::error::Error;

use super::Device;

pub mod pl011;

/// Generic serial device interface
pub trait SerialDevice: Device {
    /// Sends (blocking) a single byte into the serial port
    fn send(&self, byte: u8) -> Result<(), Error>;

    /// Receive a single byte from the serial port, blocking if necessary
    fn receive(&self, blocking: bool) -> Result<u8, Error>;
}
