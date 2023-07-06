use super::Device;

pub mod pl011;

pub trait SerialDevice: Device {
    fn send(&self, byte: u8);
}
