pub mod aarch64;

pub use aarch64::plat_qemu::{QemuPlatform as PlatformImpl, PLATFORM};
pub use aarch64::{AArch64 as ArchitectureImpl, ARCHITECTURE};
