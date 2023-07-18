use abi::error::Error;

pub trait BlockDevice {
    fn read(&self, pos: usize, buf: &mut [u8]) -> Result<(), Error>;
    fn write(&self, pos: usize, buf: &[u8]) -> Result<(), Error>;
}
