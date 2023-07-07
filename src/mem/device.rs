use core::{marker::PhantomData, mem::size_of, ops::Deref};

#[allow(unused)]
pub struct DeviceMemory {
    name: &'static str,
    base: usize,
    size: usize,
}

pub struct DeviceMemoryIo<T> {
    mmio: DeviceMemory,
    _pd: PhantomData<T>,
}

impl DeviceMemory {
    pub unsafe fn map(name: &'static str, phys: usize, size: usize) -> Self {
        if size > 0x1000 {
            todo!("Device memory mappings larger than 4K");
        }

        use crate::arch::aarch64::table::KERNEL_TABLES;
        let base = KERNEL_TABLES.map_4k(phys);

        Self { name, base, size }
    }
}

impl<T> DeviceMemoryIo<T> {
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
