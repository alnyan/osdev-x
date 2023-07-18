use core::cell::RefCell;

use abi::error::Error;
use alloc::rc::Rc;
use bitflags::bitflags;

use crate::{
    node::{VnodeKind, VnodeRef},
    Read, Write,
};

bitflags! {
    pub struct FileFlags: u32 {
        const READ = 1 << 0;
        const WRITE = 1 << 1;
    }
}

pub type FileRef = Rc<RefCell<File>>;

pub struct NormalFile {
    vnode: VnodeRef,
    pos: usize,
}

pub enum FileInner {
    Normal(NormalFile),
}

pub struct File {
    inner: FileInner,
    flags: FileFlags,
}

impl File {
    pub fn normal(vnode: VnodeRef, pos: usize, flags: FileFlags) -> FileRef {
        Rc::new(RefCell::new(Self {
            inner: FileInner::Normal(NormalFile { vnode, pos }),
            flags,
        }))
    }
}

impl Write for File {
    fn write(&mut self, data: &[u8]) -> Result<usize, Error> {
        if !self.flags.contains(FileFlags::WRITE) {
            panic!();
        }

        match &mut self.inner {
            FileInner::Normal(inner) => {
                let count = inner.vnode.write(inner.pos, data)?;
                if inner.vnode.kind() != VnodeKind::Char {
                    inner.pos += count;
                }
                Ok(count)
            }
        }
    }
}

impl Read for File {
    fn read(&mut self, data: &mut [u8]) -> Result<usize, Error> {
        if !self.flags.contains(FileFlags::READ) {
            panic!();
        }

        match &mut self.inner {
            FileInner::Normal(inner) => {
                let count = inner.vnode.read(inner.pos, data)?;
                if inner.vnode.kind() != VnodeKind::Char {
                    inner.pos += count;
                }
                Ok(count)
            }
        }
    }
}

impl Drop for File {
    fn drop(&mut self) {
        match &mut self.inner {
            FileInner::Normal(inner) => {
                inner.vnode.close().ok();
            }
        }
    }
}
