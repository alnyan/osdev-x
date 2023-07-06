pub const KERNEL_VIRT_OFFSET: usize = 0xFFFFFF8000000000;

pub unsafe trait Virtualize {
    unsafe fn virtualize(self) -> Self;
}

unsafe impl Virtualize for usize {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        #[cfg(debug_assertions)]
        if self > KERNEL_VIRT_OFFSET {
            todo!();
        }

        self + KERNEL_VIRT_OFFSET
    }
}

unsafe impl<T> Virtualize for *mut T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }
}

unsafe impl<T> Virtualize for *const T {
    #[inline(always)]
    unsafe fn virtualize(self) -> Self {
        (self as usize).virtualize() as Self
    }
}
