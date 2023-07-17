use core::time::Duration;

use abi::error::Error;

use super::Device;

pub trait TimestampSource: Device {
    fn timestamp(&self) -> Result<Duration, Error>;
}
