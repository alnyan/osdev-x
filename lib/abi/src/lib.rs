#![no_std]
use enum_repr::EnumRepr;

pub mod error;

#[EnumRepr(type = "usize")]
#[derive(Clone, Copy, Debug)]
pub enum SyscallFunction {
    DoSomething = 1,
}

pub trait SyscallArgument: Sized {
    fn as_syscall_argument(self) -> usize;
    fn from_syscall_argument(a: usize) -> Result<Self, ()>;
}

macro_rules! impl_syscall_argument {
    () => {
        #[inline(always)]
        fn as_syscall_argument(self) -> usize {
            self as usize
        }

        #[inline(always)]
        fn from_syscall_argument(a: usize) -> Result<Self, ()> {
            Ok(a as Self)
        }
    };
}

macro_rules! primitive_syscall_impl {
    ($t:ty) => {
        impl SyscallArgument for $t {
            impl_syscall_argument!();
        }
    };
}

primitive_syscall_impl!(u8);
primitive_syscall_impl!(u16);
primitive_syscall_impl!(u32);
primitive_syscall_impl!(u64);
primitive_syscall_impl!(usize);

primitive_syscall_impl!(i8);
primitive_syscall_impl!(i16);
primitive_syscall_impl!(i32);
primitive_syscall_impl!(i64);
primitive_syscall_impl!(isize);

impl<T> SyscallArgument for *const T {
    impl_syscall_argument!();
}

impl<T> SyscallArgument for *mut T {
    impl_syscall_argument!();
}
