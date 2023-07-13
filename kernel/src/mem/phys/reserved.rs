//! Utilities for handling reserved memory regions

use crate::util::StaticVector;

use super::PhysicalMemoryRegion;

static mut RESERVED_MEMORY: StaticVector<PhysicalMemoryRegion, 4> = StaticVector::new();

/// Marks a region of physical memory as reserved.
///
/// # Safety
///
/// Can only be called from initialization code **before** physical memory manager is initialized.
pub unsafe fn reserve_region(reason: &str, region: PhysicalMemoryRegion) {
    debugln!(
        "Reserve {:?} memory: {:#x}..{:#x}",
        reason,
        region.base,
        region.end()
    );

    RESERVED_MEMORY.push(region);
}

/// Returns `true` if `addr` refers to any reserved memory region
pub fn is_reserved(addr: usize) -> bool {
    for region in unsafe { RESERVED_MEMORY.iter() } {
        if region.range().contains(&addr) {
            return true;
        }
    }
    false
}
