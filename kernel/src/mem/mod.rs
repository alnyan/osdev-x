//! Memory management utilities and types
use crate::{
    arch::{Architecture, ArchitectureImpl, PlatformImpl},
    device::platform::Platform,
};

pub mod device;
pub mod heap;
pub mod phys;
pub mod table;

/// Kernel's physical load address
pub const KERNEL_PHYS_BASE: usize = PlatformImpl::KERNEL_PHYS_BASE;
/// Kernel's virtual memory mapping offset (i.e. kernel's virtual address is [KERNEL_PHYS_BASE] +
/// [KERNEL_VIRT_OFFSET])
pub const KERNEL_VIRT_OFFSET: usize = ArchitectureImpl::KERNEL_VIRT_OFFSET;

/// Interface for converting between address spaces.
///
/// # Safety
///
/// An incorrect implementation can produce invalid address.
pub unsafe trait ConvertAddress {
    /// Convert the address into a virtual one
    ///
    /// # Panics
    ///
    /// Panics if the address is already a virtual one
    ///
    /// # Safety
    ///
    /// An incorrect implementation can produce invalid address.
    unsafe fn virtualize(self) -> Self;
    /// Convert the address into a physical one
    ///
    /// # Panics
    ///
    /// Panics if the address is already a physical one
    ///
    /// # Safety
    ///
    /// An incorrect implementation can produce invalid address.
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
