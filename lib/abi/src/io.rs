use core::fmt;

#[derive(Clone, Copy, PartialEq, Debug, PartialOrd, Ord, Eq)]
pub struct RawFd(pub u32);

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct OpenFlags(pub u32);

const O_READ: u32 = 1 << 0;
const O_WRITE: u32 = 1 << 1;

impl RawFd {
    pub const STDOUT: Self = Self(1);
    pub const STDERR: Self = Self(2);
}

impl OpenFlags {
    pub fn new() -> Self {
        Self(0)
    }

    pub const fn read(mut self) -> Self {
        self.0 |= O_READ;
        self
    }

    pub const fn write(mut self) -> Self {
        self.0 |= O_WRITE;
        self
    }

    pub const fn is_read(self) -> bool {
        self.0 & O_READ != 0
    }

    pub const fn is_write(self) -> bool {
        self.0 & O_WRITE != 0
    }
}

impl fmt::Debug for OpenFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OpenFlags")
            .field("read", &(self.is_read()))
            .field("write", &(self.is_write()))
            .finish()
    }
}
