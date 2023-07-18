//! Device virtual file system
use core::sync::atomic::{AtomicUsize, Ordering};

use abi::error::Error;
use alloc::{boxed::Box, format, string::String};
use vfs::{CharDevice, CharDeviceWrapper, Vnode, VnodeKind, VnodeRef};

use crate::util::OneTimeInit;

/// Describes the kind of a character device
#[derive(Debug)]
pub enum CharDeviceType {
    /// Serial terminal
    TtySerial,
}

static DEVFS_ROOT: OneTimeInit<VnodeRef> = OneTimeInit::new();

/// Sets up the device filesystem
pub fn init() {
    let node = Vnode::new("", VnodeKind::Directory);
    DEVFS_ROOT.init(node);
}

/// Returns the root of the devfs.
///
/// # Panics
///
/// Will panic if the devfs hasn't yet been initialized.
pub fn root() -> &'static VnodeRef {
    DEVFS_ROOT.get()
}

fn _add_char_device(dev: &'static dyn CharDevice, name: String) -> Result<(), Error> {
    infoln!("Add char device: {}", name);

    let node = Vnode::new(name, VnodeKind::Char);
    node.set_data(Box::new(CharDeviceWrapper::new(dev)));

    DEVFS_ROOT.get().add_child(node);

    Ok(())
}

/// Adds a character device to the devfs
pub fn add_char_device(dev: &'static dyn CharDevice, kind: CharDeviceType) -> Result<(), Error> {
    static TTYS_COUNT: AtomicUsize = AtomicUsize::new(0);

    let (count, prefix) = match kind {
        CharDeviceType::TtySerial => (&TTYS_COUNT, "ttyS"),
    };

    let value = count.fetch_add(1, Ordering::AcqRel);
    let name = format!("{}{}", prefix, value);

    _add_char_device(dev, name)
}
