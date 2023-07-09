//! Physical memory management facilities
use core::{iter::StepBy, mem::size_of, ops::Range};

use crate::{
    absolute_address,
    mem::{
        phys::reserved::{is_reserved, reserve_region},
        KERNEL_PHYS_BASE,
    },
    util::{OneTimeInit, SpinLock},
};

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

/// Physical memory management interface
pub struct PhysicalMemoryManager {
    pages: &'static mut [Page],
    offset: usize,
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

impl PhysicalMemoryManager {
    /// Constructs a [PhysicalMemoryManager] with page tracking array placed at given
    /// `base`..`base+size` range. Physical addresses allocated are offset by the given value.
    ///
    /// # Safety
    ///
    /// Addresses are not checked. The caller is responsible for making sure (base, size) ranges do
    /// not alias/overlap, they're accessible through virtual memory and that the offset is a
    /// meaningful value.
    pub unsafe fn new(offset: usize, base: usize, size: usize) -> PhysicalMemoryManager {
        // TODO check alignment
        let page_count = size / size_of::<Page>();
        let pages = core::slice::from_raw_parts_mut(base as *mut _, page_count);

        for page in pages.iter_mut() {
            *page = Page {
                usage: PageUsage::Reserved,
                refcount: 0,
            };
        }

        PhysicalMemoryManager { pages, offset }
    }

    /// Allocates a single page, marking it as used with `usage`.
    pub fn alloc_page(&mut self, usage: PageUsage) -> usize {
        assert_ne!(usage, PageUsage::Available);
        assert_ne!(usage, PageUsage::Reserved);

        for index in 0..self.pages.len() {
            if self.pages[index].usage == PageUsage::Available {
                self.pages[index].usage = PageUsage::Used;
                return index * 4096 + self.offset;
            }
        }

        panic!();
    }

    /// Marks a previously reserved page as available.
    ///
    /// # Panics
    ///
    /// Will panic if the address does not point to a valid, reserved (and unallocated) page.
    pub fn add_available_page(&mut self, addr: usize) {
        assert!(addr >= self.offset);
        let index = (addr - self.offset) / 4096;

        assert_eq!(self.pages[index].usage, PageUsage::Reserved);
        assert_eq!(self.pages[index].refcount, 0);

        self.pages[index].usage = PageUsage::Available;
    }
}

/// Global physical memory manager
pub static PHYSICAL_MEMORY: OneTimeInit<SpinLock<PhysicalMemoryManager>> = OneTimeInit::new();

/// Allocates a single physical page from the global manager
pub fn alloc_page(usage: PageUsage) -> usize {
    PHYSICAL_MEMORY.get().lock().alloc_page(usage)
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

    PHYSICAL_MEMORY.init(SpinLock::new(manager));
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