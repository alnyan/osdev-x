//! Kernel's global heap allocator
use core::{
    alloc::{GlobalAlloc, Layout},
    ptr::{null_mut, NonNull},
};

use linked_list_allocator::Heap;
use spinning_top::Spinlock;

struct KernelAllocator {
    inner: Spinlock<Heap>,
}

impl KernelAllocator {
    const fn empty() -> Self {
        Self {
            inner: Spinlock::new(Heap::empty()),
        }
    }

    unsafe fn init(&self, base: usize, size: usize) {
        self.inner.lock().init(base as _, size);
    }
}

unsafe impl GlobalAlloc for KernelAllocator {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        debugln!("alloc {:?}", layout);
        match self.inner.lock().allocate_first_fit(layout) {
            Ok(v) => v.as_ptr(),
            Err(_) => null_mut(),
        }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let ptr = NonNull::new(ptr).unwrap();
        self.inner.lock().deallocate(ptr, layout)
    }
}

#[global_allocator]
static GLOBAL_HEAP: KernelAllocator = KernelAllocator::empty();

/// Sets up kernel's global heap with given memory range.
///
/// # Safety
///
/// The caller must ensure the range is valid and mapped virtual memory.
pub unsafe fn init_heap(heap_base: usize, heap_size: usize) {
    debugln!("Heap: {:#x}..{:#x}", heap_base, heap_base + heap_size);
    GLOBAL_HEAP.init(heap_base, heap_size);
}
