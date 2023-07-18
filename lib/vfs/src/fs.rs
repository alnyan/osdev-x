use core::{any::Any, cell::Ref};

use alloc::rc::Rc;

use abi::error::Error;

use crate::{block::BlockDevice, node::VnodeRef};

pub trait Filesystem {
    fn root(self: Rc<Self>) -> Result<VnodeRef, Error>;

    fn dev(self: Rc<Self>) -> Option<&'static dyn BlockDevice>;

    fn data(&self) -> Option<Ref<dyn Any>>;
}
