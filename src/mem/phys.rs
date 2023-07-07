use core::mem::size_of;

#[derive(PartialEq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum PageUsage {
    Reserved = 0,
    Available,
    Used,
}

#[repr(C)]
pub struct Page {
    usage: PageUsage,
    refcount: u32,
}

pub struct PhysicalMemoryManager {
    pages: &'static mut [Page],
    offset: usize,
}

impl PhysicalMemoryManager {
    pub unsafe fn new(offset: usize, base: usize, size: usize) -> PhysicalMemoryManager {
        // TODO check alignment
        let page_count = size / size_of::<Page>();
        let pages = core::slice::from_raw_parts_mut(base as *mut _, page_count);

        for i in 0..page_count {
            pages[i] = Page {
                usage: PageUsage::Reserved,
                refcount: 0,
            };
        }

        PhysicalMemoryManager { pages, offset }
    }

    pub fn alloc_page(&mut self) -> usize {
        for index in 0..self.pages.len() {
            if self.pages[index].usage == PageUsage::Available {
                self.pages[index].usage = PageUsage::Used;
                return index * 4096 + self.offset;
            }
        }

        panic!();
    }

    pub fn add_available_page(&mut self, addr: usize) {
        assert!(addr > self.offset);
        let index = (addr - self.offset) / 4096;

        assert_eq!(self.pages[index].usage, PageUsage::Reserved);
        assert_eq!(self.pages[index].refcount, 0);

        self.pages[index].usage = PageUsage::Available;
    }
}
