//! Physical memory manager implementation
use core::mem::size_of;

use super::{Page, PageUsage};

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

    /// Allocates a single page, marking it as used with `usage`
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

    /// Allocates a contiguous range of physical pages, marking it as used with `usage`
    pub fn alloc_contiguous_pages(&mut self, count: usize, usage: PageUsage) -> usize {
        assert_ne!(usage, PageUsage::Available);
        assert_ne!(usage, PageUsage::Reserved);
        assert_ne!(count, 0);

        'l0: for i in 0..self.pages.len() {
            for j in 0..count {
                if self.pages[i + j].usage != PageUsage::Available {
                    continue 'l0;
                }
            }
            for j in 0..count {
                let page = &mut self.pages[i + j];
                assert!(page.usage == PageUsage::Available);
                page.usage = usage;
                page.refcount = 1;
            }
            return self.offset + i * 0x1000;
        }

        panic!("Out of memory");
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
