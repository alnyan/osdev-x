#![no_std]
use enum_repr::EnumRepr;

pub mod error;

#[EnumRepr(type = "usize")]
#[derive(Clone, Copy, Debug)]
pub enum SyscallFunction {
    Exit = 1,

    DebugTrace = 128,
}
