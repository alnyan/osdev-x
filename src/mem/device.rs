//! Facilities for mapping devices to virtual address space
use core::{marker::PhantomData, mem::size_of, ops::Deref};

/// Generic MMIO access mapping
#[allow(unused)]
pub struct DeviceMemory {
    name: &'static str,
    base: usize,
    size: usize,
}

/// MMIO wrapper for `T`
pub struct DeviceMemoryIo<T> {
    mmio: DeviceMemory,
    _pd: PhantomData<T>,
}

impl DeviceMemory {
    /// Maps the device to some virtual memory address and constructs a wrapper for that range.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure the (phys, size) range is valid and actually
    /// points to some device's MMIO. The caller must also make sure no aliasing for that range is
    /// possible.
    pub unsafe fn map(name: &'static str, phys: usize, size: usize) -> Self {
        if size > 0x1000 {
            todo!("Device memory mappings larger than 4K");
        }

        use crate::arch::aarch64::table::KERNEL_TABLES;
        let base = KERNEL_TABLES.map_device_4k(phys);

        Self { name, base, size }
    }
}

impl<T> DeviceMemoryIo<T> {
    /// Maps the `T` struct at `phys` to some virtual memory address and provides a [Deref]able
    /// wrapper to it.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure the `phys` address points to a MMIO region which
    /// is at least `size_of::<T>()` and no aliasing for that region is possible.
    pub unsafe fn map(name: &'static str, phys: usize) -> Self {
        Self {
            mmio: DeviceMemory::map(name, phys, size_of::<T>()),
            _pd: PhantomData,
        }
    }
}

impl<T> Deref for DeviceMemoryIo<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        unsafe { &*(self.mmio.base as *const T) }
    }
}
