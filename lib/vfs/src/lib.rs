#![no_std]

use abi::error::Error;

extern crate alloc;

#[cfg(test)]
extern crate std;

pub(crate) mod block;
pub(crate) mod char;
pub(crate) mod file;
pub(crate) mod fs;
pub(crate) mod ioctx;
pub(crate) mod node;

pub use self::block::BlockDevice;
pub use self::char::{CharDevice, CharDeviceWrapper};
pub use file::{File, FileFlags, FileRef};
pub use ioctx::IoContext;
pub use node::{Vnode, VnodeImpl, VnodeKind, VnodeRef, VnodeWeak};

pub trait Write {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error>;
}

pub trait Read {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, Error>;
}
