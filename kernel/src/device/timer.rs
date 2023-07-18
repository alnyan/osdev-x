//! Time-providing device interfaces
use core::time::Duration;

use abi::error::Error;

use super::Device;

/// Interface for devices capable of providing some notion of time
pub trait TimestampSource: Device {
    /// Returns current time signalled by the device. The time may not be a "real" time and instead
    /// is assumed to be monotonically increasing.
    fn timestamp(&self) -> Result<Duration, Error>;
}
