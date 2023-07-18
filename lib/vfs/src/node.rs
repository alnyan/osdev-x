use core::{
    cell::{RefCell, RefMut},
    fmt,
};

use abi::{error::Error, io::OpenFlags};
use alloc::{
    boxed::Box,
    rc::{Rc, Weak},
    string::String,
    vec::Vec,
};

use crate::file::{File, FileFlags, FileRef};

pub type VnodeRef = Rc<Vnode>;
pub type VnodeWeak = Weak<Vnode>;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum VnodeKind {
    Directory,
    Regular,
    Char,
    Block,
}

pub(crate) struct TreeNode {
    parent: Option<VnodeWeak>,
    children: Vec<VnodeRef>,
}

pub struct Vnode {
    name: String,
    tree: RefCell<TreeNode>,
    kind: VnodeKind,
    data: RefCell<Option<Box<dyn VnodeImpl>>>,
}

pub trait VnodeImpl {
    fn create(&mut self, at: &VnodeRef, name: &str, kind: VnodeKind) -> Result<VnodeRef, Error>;

    fn open(&mut self, node: &VnodeRef, opts: OpenFlags) -> Result<usize, Error>;
    fn close(&mut self, node: &VnodeRef) -> Result<(), Error>;

    fn read(&mut self, node: &VnodeRef, pos: usize, data: &mut [u8]) -> Result<usize, Error>;
    fn write(&mut self, node: &VnodeRef, pos: usize, data: &[u8]) -> Result<usize, Error>;
}

impl Vnode {
    pub fn new<S: Into<String>>(name: S, kind: VnodeKind) -> VnodeRef {
        Rc::new(Self {
            name: name.into(),
            tree: RefCell::new(TreeNode {
                parent: None,
                children: Vec::new(),
            }),
            kind,
            data: RefCell::new(None),
        })
    }

    #[inline]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline]
    pub fn kind(&self) -> VnodeKind {
        self.kind
    }

    #[inline]
    pub fn data(&self) -> RefMut<Option<Box<dyn VnodeImpl>>> {
        self.data.borrow_mut()
    }

    pub fn parent(self: &VnodeRef) -> VnodeRef {
        match &self.tree.borrow().parent {
            Some(parent) => parent.upgrade().unwrap(),
            None => self.clone(),
        }
    }

    pub fn set_data(&self, data: Box<dyn VnodeImpl>) {
        self.data.borrow_mut().replace(data);
    }

    #[inline]
    pub fn is_directory(&self) -> bool {
        self.kind == VnodeKind::Directory
    }

    // Cache tree operations
    pub fn add_child(self: &VnodeRef, child: VnodeRef) {
        let parent_weak = Rc::downgrade(self);
        let mut parent_borrow = self.tree.borrow_mut();

        assert!(child
            .tree
            .borrow_mut()
            .parent
            .replace(parent_weak)
            .is_none());
        parent_borrow.children.push(child);
    }

    pub fn dump(&self, f: &mut fmt::Formatter<'_>, depth: usize) -> fmt::Result {
        for _ in 0..depth {
            f.write_str("  ")?;
        }

        write!(f, "{:?}", self.name)?;

        match self.kind {
            VnodeKind::Directory => {
                let tree = self.tree.borrow();
                if tree.children.is_empty() {
                    f.write_str(" []")?;
                } else {
                    f.write_str(" [\n")?;
                    for child in tree.children.iter() {
                        child.dump(f, depth + 1)?;
                        f.write_str("\n")?;
                    }
                    for _ in 0..depth {
                        f.write_str("  ")?;
                    }
                    f.write_str("]")?;
                }
            }
            _ => (),
        }

        Ok(())
    }

    pub fn lookup(self: &VnodeRef, name: &str) -> Option<VnodeRef> {
        assert!(self.is_directory());
        self.tree
            .borrow()
            .children
            .iter()
            .find(|e| e.name == name)
            .cloned()
    }

    //
    pub fn lookup_or_load(self: &VnodeRef, name: &str) -> Result<VnodeRef, Error> {
        // Lookup in cache
        if let Some(node) = self.lookup(name) {
            return Ok(node);
        }

        // TODO load from FS
        Err(Error::DoesNotExist)
    }

    // Node operations
    pub fn open(self: &VnodeRef, flags: OpenFlags) -> Result<FileRef, Error> {
        let mut open_flags = FileFlags::empty();

        if flags.is_read() {
            open_flags |= FileFlags::READ;
        }
        if flags.is_write() {
            open_flags |= FileFlags::WRITE;
        }

        if self.kind == VnodeKind::Directory {
            return Err(Error::IsADirectory);
        }

        if let Some(ref mut data) = *self.data() {
            let pos = data.open(self, flags)?;
            Ok(File::normal(self.clone(), pos, open_flags))
        } else {
            todo!()
        }
    }

    pub fn close(self: &VnodeRef) -> Result<(), Error> {
        if let Some(ref mut data) = *self.data() {
            data.close(self)
        } else {
            todo!()
        }
    }

    pub fn write(self: &VnodeRef, pos: usize, buf: &[u8]) -> Result<usize, Error> {
        if self.kind == VnodeKind::Directory {
            todo!();
        }

        if let Some(ref mut data) = *self.data() {
            data.write(self, pos, buf)
        } else {
            todo!()
        }
    }

    pub fn read(self: &VnodeRef, pos: usize, buf: &mut [u8]) -> Result<usize, Error> {
        if self.kind == VnodeKind::Directory {
            todo!();
        }

        if let Some(ref mut data) = *self.data() {
            data.read(self, pos, buf)
        } else {
            todo!()
        }
    }
}

impl fmt::Debug for Vnode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let prefix = match self.kind {
            VnodeKind::Directory => "DIR ",
            VnodeKind::Regular => "REG ",
            VnodeKind::Char => "CHR ",
            VnodeKind::Block => "BLK ",
        };

        write!(f, "[{} {}]", prefix, self.name)
    }
}
