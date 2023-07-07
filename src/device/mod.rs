use self::serial::SerialDevice;

pub mod serial;

pub trait Device {
    unsafe fn init(&self);
    fn name(&self) -> &'static str;
}

pub trait Platform {
    const KERNEL_PHYS_BASE: usize;

    unsafe fn init(&self);
    unsafe fn init_primary_serial(&self);

    fn name(&self) -> &'static str;
    fn primary_serial(&self) -> Option<&dyn SerialDevice>;
}

pub trait Architecture {
    const KERNEL_VIRT_OFFSET: usize;

    unsafe fn init_mmu(&self);
}
