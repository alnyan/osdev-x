//! Physical memory management facilities
use core::mem::size_of;

use crate::util::{OneTimeInit, SpinLock};

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
#[derive(Clone)]
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

/// Sets up physical memory manager with given memory ranges, placing page tracking array at
/// `(pages_array_base, pages_array_size)` range.
///
/// # Safety
///
/// Unsafe to call if the physical memory manager has already been initialized. The caller is
/// responsible for making sure memory regions are valid usable memory and that the page tracking
/// array range addresses are valid and accessible through virtual memory.
pub unsafe fn init_with_array<I: Iterator<Item = PhysicalMemoryRegion> + Clone>(
    it: I,
    pages_array_base: usize,
    pages_array_size: usize,
) {
    let (phys_start, phys_end) = physical_memory_range(it.clone()).unwrap();
    let max_pages = pages_array_size / size_of::<Page>();
    let total_pages = core::cmp::min((phys_end - phys_start) / 0x1000, max_pages);

    assert!(total_pages > 0);

    let mut phys = PhysicalMemoryManager::new(phys_start, pages_array_base, pages_array_size);

    debugln!(
        "Page manager array in {:#x}..{:#x}",
        pages_array_base,
        pages_array_base + pages_array_size
    );

    // TODO reserve kernel/initrd/DTB pages

    let mut available_pages = 0;
    for reg in it {
        debugln!("Available: {:#x}..{:#x}", reg.base, reg.base + reg.size);
        for page in (0..reg.size).step_by(0x1000) {
            phys.add_available_page(reg.base + page);
            available_pages += 1;
        }
    }

    debugln!("{} pages available", available_pages);

    PHYSICAL_MEMORY.init(SpinLock::new(phys));
}
