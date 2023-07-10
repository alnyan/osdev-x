//! Physical memory management facilities
use core::{iter::StepBy, mem::size_of, ops::Range};

use spinning_top::Spinlock;

use crate::{
    absolute_address,
    mem::{
        phys::reserved::{is_reserved, reserve_region},
        KERNEL_PHYS_BASE,
    },
    util::OneTimeInit,
};

use self::manager::PhysicalMemoryManager;

pub mod manager;
pub mod reserved;

/// Represents the way in which the page is used (or not)
#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum PageUsage {
    /// Page is not available for allocation or use
    Reserved = 0,
    /// Regular page available for allocation
    Available,
    /// Page is used by some kernel facility
    Used,
}

/// Page descriptor structure for the page management array
#[repr(C)]
pub struct Page {
    usage: PageUsage,
    refcount: u32,
}

/// Defines an usable memory region
#[derive(Clone, Copy, Debug)]
pub struct PhysicalMemoryRegion {
    /// Start of the region
    pub base: usize,
    /// Length of the region
    pub size: usize,
}

impl PhysicalMemoryRegion {
    /// Returns the end address of the region
    pub const fn end(&self) -> usize {
        self.base + self.size
    }

    /// Returns an address range covered by the region
    pub const fn range(&self) -> Range<usize> {
        self.base..self.end()
    }

    /// Provides an iterator over the pages in the region
    pub const fn pages(&self) -> StepBy<Range<usize>> {
        self.range().step_by(0x1000)
    }
}

/// Global physical memory manager
pub static PHYSICAL_MEMORY: OneTimeInit<Spinlock<PhysicalMemoryManager>> = OneTimeInit::new();

/// Allocates a single physical page from the global manager
pub fn alloc_page(usage: PageUsage) -> usize {
    PHYSICAL_MEMORY.get().lock().alloc_page(usage)
}

/// Allocates a contiguous range of physical pages from the global manager
pub fn alloc_pages_contiguous(count: usize, usage: PageUsage) -> usize {
    PHYSICAL_MEMORY
        .get()
        .lock()
        .alloc_contiguous_pages(count, usage)
}

fn physical_memory_range<I: Iterator<Item = PhysicalMemoryRegion>>(
    it: I,
) -> Option<(usize, usize)> {
    let mut start = usize::MAX;
    let mut end = usize::MIN;

    for reg in it {
        if reg.base < start {
            start = reg.base;
        }
        if reg.base + reg.size > end {
            end = reg.base + reg.size;
        }
    }

    if start == usize::MAX || end == usize::MIN {
        None
    } else {
        Some((start, end))
    }
}

fn find_contiguous_region<I: Iterator<Item = PhysicalMemoryRegion>>(
    it: I,
    count: usize,
) -> Option<usize> {
    for region in it {
        let mut collected = 0;
        let mut base_addr = None;

        for addr in region.pages() {
            if is_reserved(addr) {
                collected = 0;
                base_addr = None;
                continue;
            }
            if base_addr.is_none() {
                base_addr = Some(addr);
            }
            collected += 1;
            if collected == count {
                return base_addr;
            }
        }
    }
    todo!()
}

/// Initializes physical memory manager from given available memory region iterator.
///
/// 1. Finds a non-reserved range to place the page tracking array.
/// 2. Adds all non-reserved pages to the manager.
///
/// # Safety
///
/// The caller must ensure this function has not been called before and that the regions
/// are valid and actually available.
pub unsafe fn init_from_iter<I: Iterator<Item = PhysicalMemoryRegion> + Clone>(it: I) {
    let (phys_start, phys_end) = physical_memory_range(it.clone()).unwrap();
    let total_count = (phys_end - phys_start) / 0x1000;
    let pages_array_size = total_count * size_of::<Page>();

    debugln!("Initializing physical memory manager");
    debugln!("Total tracked pages: {}", total_count);

    // Reserve memory regions from which allocation is forbidden
    reserve_region("kernel", kernel_physical_memory_region());

    let pages_array_base =
        find_contiguous_region(it.clone(), (pages_array_size + 0xFFF) / 0x1000).unwrap();
    debugln!("Placing page tracking at {:#x}", pages_array_base);

    let mut manager = PhysicalMemoryManager::new(phys_start, pages_array_base, pages_array_size);
    let mut page_count = 0;

    for region in it {
        for page in region.pages() {
            if is_reserved(page) {
                continue;
            }

            manager.add_available_page(page);
            page_count += 1;
        }
    }

    infoln!("{} available pages", page_count);

    PHYSICAL_MEMORY.init(Spinlock::new(manager));
}

fn kernel_physical_memory_region() -> PhysicalMemoryRegion {
    extern "C" {
        static __kernel_size: u8;
    }
    let size = absolute_address!(__kernel_size);

    PhysicalMemoryRegion {
        base: KERNEL_PHYS_BASE,
        size,
    }
}
