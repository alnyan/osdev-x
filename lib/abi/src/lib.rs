#![no_std]

pub mod error;
pub mod io;
pub mod path;

#[derive(Clone, Copy, Debug)]
pub enum SyscallFunction {
    Exit = 1,
    Nanosleep = 2,
    MapMemory = 3,
    UnmapMemory = 4,
    Write = 5,
    Read = 6,
    Open = 7,
    Close = 8,

    DebugTrace = 128,
}

impl TryFrom<usize> for SyscallFunction {
    type Error = ();

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            1 => Ok(Self::Exit),
            2 => Ok(Self::Nanosleep),
            3 => Ok(Self::MapMemory),
            4 => Ok(Self::UnmapMemory),
            5 => Ok(Self::Write),
            6 => Ok(Self::Read),
            7 => Ok(Self::Open),
            8 => Ok(Self::Close),

            128 => Ok(Self::DebugTrace),

            _ => Err(()),
        }
    }
}

impl From<SyscallFunction> for usize {
    fn from(value: SyscallFunction) -> Self {
        match value {
            SyscallFunction::Exit => 1,
            SyscallFunction::Nanosleep => 2,
            SyscallFunction::MapMemory => 3,
            SyscallFunction::UnmapMemory => 4,
            SyscallFunction::Write => 5,
            SyscallFunction::Read => 6,
            SyscallFunction::Open => 7,
            SyscallFunction::Close => 8,

            SyscallFunction::DebugTrace => 128,
        }
    }
}
