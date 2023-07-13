use enum_repr::EnumRepr;

#[EnumRepr(type = "u32", implicit = true)]
#[derive(Clone, Copy, Debug)]
pub enum Error {
    OutOfMemory = 1,
    InvalidMemoryOperation,
    AlreadyExists,
}
