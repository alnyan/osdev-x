use crate::io::RawFd;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Error {
    OutOfMemory = 1,
    InvalidMemoryOperation,
    AlreadyExists,
    TimedOut,
    InvalidArgument,
    DoesNotExist,
    IsADirectory,
    InvalidFile,
}

pub trait FromSyscallResult: Sized {
    fn from_syscall_result(value: usize) -> Result<Self, Error>;
}

pub trait IntoSyscallResult {
    fn into_syscall_result(self) -> usize;
}

pub trait SyscallError {
    fn from_syscall_error(value: usize) -> Self;
    fn into_syscall_error(self) -> usize;
}

impl TryFrom<u32> for Error {
    type Error = ();

    fn try_from(value: u32) -> Result<Self, ()> {
        match value {
            1 => Ok(Self::OutOfMemory),
            2 => Ok(Self::InvalidMemoryOperation),
            3 => Ok(Self::AlreadyExists),
            4 => Ok(Self::TimedOut),
            5 => Ok(Self::InvalidArgument),
            6 => Ok(Self::DoesNotExist),
            7 => Ok(Self::IsADirectory),
            8 => Ok(Self::InvalidFile),

            _ => Err(()),
        }
    }
}

impl From<Error> for u32 {
    fn from(value: Error) -> Self {
        match value {
            Error::OutOfMemory => 1,
            Error::InvalidMemoryOperation => 2,
            Error::AlreadyExists => 3,
            Error::TimedOut => 4,
            Error::InvalidArgument => 5,
            Error::DoesNotExist => 6,
            Error::IsADirectory => 7,
            Error::InvalidFile => 8,
        }
    }
}

impl SyscallError for Error {
    fn from_syscall_error(value: usize) -> Self {
        Error::try_from((-(value as isize)) as u32).unwrap_or(Error::InvalidArgument)
    }

    fn into_syscall_error(self) -> usize {
        (-((self as u32) as isize)) as usize
    }
}

impl<T: IntoSyscallResult> IntoSyscallResult for Result<T, Error> {
    fn into_syscall_result(self) -> usize {
        match self {
            Ok(t) => t.into_syscall_result(),
            Err(e) => e.into_syscall_error(),
        }
    }
}

impl FromSyscallResult for () {
    fn from_syscall_result(value: usize) -> Result<Self, Error> {
        if (value as isize) < 0 {
            Err(Error::from_syscall_error(value))
        } else {
            // TODO assert value == 0
            Ok(())
        }
    }
}

impl IntoSyscallResult for () {
    fn into_syscall_result(self) -> usize {
        0
    }
}

impl FromSyscallResult for usize {
    fn from_syscall_result(value: usize) -> Result<Self, Error> {
        if (value as isize) < 0 {
            Err(Error::from_syscall_error(value))
        } else {
            Ok(value)
        }
    }
}

impl IntoSyscallResult for usize {
    fn into_syscall_result(self) -> usize {
        assert!((self as isize) > 0);
        self
    }
}

impl FromSyscallResult for RawFd {
    fn from_syscall_result(value: usize) -> Result<Self, Error> {
        if (value as isize) < 0 {
            Err(Error::from_syscall_error(value))
        } else {
            // TODO assert value < u32::MAX
            Ok(RawFd(value as u32))
        }
    }
}

impl IntoSyscallResult for RawFd {
    fn into_syscall_result(self) -> usize {
        self.0 as usize
    }
}
