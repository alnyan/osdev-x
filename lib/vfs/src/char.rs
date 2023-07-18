use abi::error::Error;

use crate::node::{VnodeImpl, VnodeRef};

pub trait CharDevice {
    fn read(&'static self, blocking: bool, data: &mut [u8]) -> Result<usize, Error>;
    fn write(&self, blocking: bool, data: &[u8]) -> Result<usize, Error>;
}

pub struct CharDeviceWrapper {
    device: &'static dyn CharDevice,
}

impl CharDeviceWrapper {
    pub const fn new(device: &'static dyn CharDevice) -> Self {
        Self { device }
    }
}

impl VnodeImpl for CharDeviceWrapper {
    fn open(&mut self, _node: &VnodeRef, _opts: abi::io::OpenFlags) -> Result<usize, Error> {
        Ok(0)
    }

    fn close(&mut self, _node: &VnodeRef) -> Result<(), Error> {
        Ok(())
    }

    fn read(&mut self, _node: &VnodeRef, _pos: usize, data: &mut [u8]) -> Result<usize, Error> {
        self.device.read(true, data)
    }

    fn write(&mut self, _node: &VnodeRef, _pos: usize, data: &[u8]) -> Result<usize, Error> {
        self.device.write(true, data)
    }

    fn create(
        &mut self,
        _at: &VnodeRef,
        _name: &str,
        _kind: crate::node::VnodeKind,
    ) -> Result<VnodeRef, Error> {
        todo!()
    }
}
