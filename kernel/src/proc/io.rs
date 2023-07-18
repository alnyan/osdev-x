//! Process I/O management
use abi::{error::Error, io::RawFd};
use alloc::collections::BTreeMap;
use vfs::{FileRef, IoContext};

/// I/O context of a process, contains information like root, current directory and file
/// descriptor table
pub struct ProcessIo {
    ioctx: Option<IoContext>,
    files: BTreeMap<RawFd, FileRef>,
}

impl ProcessIo {
    /// Constructs an uninitialized I/O context
    pub fn new() -> Self {
        Self {
            ioctx: None,
            files: BTreeMap::new(),
        }
    }

    /// Returns a file given descriptor refers to
    pub fn file(&self, fd: RawFd) -> Result<FileRef, Error> {
        self.files
            .get(&fd)
            .cloned()
            .ok_or_else(|| Error::InvalidFile)
    }

    /// Sets the inner I/O context
    pub fn set_ioctx(&mut self, ioctx: IoContext) {
        self.ioctx.replace(ioctx);
    }

    /// Inserts a file into the descriptor table. Returns error if the file is already present for
    /// given descriptor.
    pub fn set_file(&mut self, fd: RawFd, file: FileRef) -> Result<(), Error> {
        if self.files.contains_key(&fd) {
            todo!();
        }

        self.files.insert(fd, file);
        Ok(())
    }

    /// Allocates a slot for a file and returns it
    pub fn place_file(&mut self, file: FileRef) -> Result<RawFd, Error> {
        for idx in 0..64 {
            let fd = RawFd(idx);
            if !self.files.contains_key(&fd) {
                self.files.insert(fd, file);
                return Ok(fd);
            }
        }
        todo!();
    }

    /// Closes the file and removes it from the table
    pub fn close_file(&mut self, fd: RawFd) -> Result<(), Error> {
        let file = self.files.remove(&fd);
        if file.is_none() {
            todo!();
        }
        Ok(())
    }

    /// Returns the inner I/O context reference
    pub fn ioctx(&mut self) -> &mut IoContext {
        self.ioctx.as_mut().unwrap()
    }
}
