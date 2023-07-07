use crate::{
    arch::{ArchitectureImpl, PlatformImpl},
    device::{Architecture, Platform},
};

pub mod device;
pub mod table;

pub const KERNEL_PHYS_BASE: usize = PlatformImpl::KERNEL_PHYS_BASE;
pub const KERNEL_VIRT_OFFSET: usize = ArchitectureImpl::KERNEL_VIRT_OFFSET;

pub unsafe trait ConvertAddress {
    unsafe fn virtualize(self) -> Self;
    unsafe fn physicalize(self) -> Self;
}

unsafe impl ConvertAddress for usize {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        #[cfg(debug_assertions)]
        if self > KERNEL_VIRT_OFFSET {
            todo!();
        }

        self + KERNEL_VIRT_OFFSET
    }

    #[inline(always)]
    unsafe fn physicalize(self) -> Self {
        #[cfg(debug_assertions)]
        if self < KERNEL_VIRT_OFFSET {
            todo!();
        }

        self - KERNEL_VIRT_OFFSET
    }
}

unsafe impl<T> ConvertAddress for *mut T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }

    #[inline(always)]
    unsafe fn physicalize(self) -> Self {
        (self as usize).physicalize() as Self
    }
}

unsafe impl<T> ConvertAddress for *const T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }

    #[inline(always)]
    unsafe fn physicalize(self) -> Self {
        (self as usize).physicalize() as Self
    }
}
