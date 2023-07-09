//! Facilities for mapping devices to virtual address space
use core::{marker::PhantomData, mem::size_of, ops::Deref};

use crate::{arch::ARCHITECTURE, device::Architecture};

/// Generic MMIO access mapping
#[derive(Clone)]
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

        let base = ARCHITECTURE.map_device_pages(phys, 1);

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

    /// Constructs a device MMIO wrapper from given [DeviceMemory] mapping.
    ///
    /// # Safety
    ///
    /// The caller must ensure `mmio` actually points to a device of type `T`.
    pub unsafe fn new(mmio: DeviceMemory) -> Self {
        assert!(mmio.size >= size_of::<T>());
        // TODO check align

        Self {
            mmio,
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
