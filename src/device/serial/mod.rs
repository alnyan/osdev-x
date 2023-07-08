//! Serial device interfaces
use super::Device;

pub mod pl011;

/// Generic serial device interface
pub trait SerialDevice: Device {
    /// Sends (blocking) a single byte into the serial port
    fn send(&self, byte: u8);
}
