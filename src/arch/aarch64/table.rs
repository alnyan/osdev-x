//! AArch64 virtual memory management facilities
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;
use tables::KernelTables;

use crate::mem::{
    table::{EntryLevel, NextPageTable},
    ConvertAddress, KERNEL_PHYS_BASE, KERNEL_VIRT_OFFSET,
};

/// TODO
#[derive(Clone)]
#[repr(C, align(0x1000))]
pub struct AddressSpace {
    l1: PageTable<L1>,
}

/// Page table representing a single level of address translation
#[derive(Clone)]
#[repr(C, align(0x1000))]
pub struct PageTable<L: EntryLevel> {
    data: [PageEntry<L>; 512],
}

/// Fixed struct for kernel-space address mapping (kernel/device/page tracking array mapping)
pub struct FixedTables {
    l1: PageTable<L1>,
    l2: PageTable<L2>,
    kernel_l3: PageTable<L3>,
    device_l3: PageTable<L3>,

    l1i: usize,

    kernel_l2i: usize,

    device_l2i: usize,
    device_l3i: usize,

    pages_l2i: usize,
}

/// Translation level 1: Entry is 1GiB page/table
#[derive(Clone)]
pub struct L1;
/// Translation level 2: Entry is 2MiB page/table
#[derive(Clone)]
pub struct L2;
/// Translation level 3: Entry is 4KiB page
#[derive(Clone)]
pub struct L3;

bitflags! {
    /// TODO split attrs for different translation levels
    ///
    /// Describes how each page is mapped: access, presence, type of the mapping.
    pub struct PageAttributes: u64 {
        /// When set, the mapping is considered valid and assumed to point to a page/table
        const PRESENT = 1 << 0;

        /// For L1/L2 mappings, indicates that the mapping points to the next-level translation
        /// table
        const TABLE = 1 << 1;
        /// (Must be set) For L3 mappings, indicates that the mapping points to a page
        const PAGE = 1 << 1;
        /// For L1/L2 mappings, indicates that the mapping points to a page of given level's size
        const BLOCK = 0 << 1;

        /// (Must be set) For page/block mappings, indicates to the hardware that the page is
        /// accessed
        const ACCESS = 1 << 10;
    }
}

impl const EntryLevel for L1 {
    fn index(addr: usize) -> usize {
        (addr >> 30) & 0x1FF
    }

    fn page_offset(addr: usize) -> usize {
        addr & 0x3FFFFFFF
    }
}
impl const EntryLevel for L2 {
    fn index(addr: usize) -> usize {
        (addr >> 21) & 0x1FF
    }

    fn page_offset(addr: usize) -> usize {
        addr & 0x1FFFFF
    }
}
impl const EntryLevel for L3 {
    fn index(addr: usize) -> usize {
        (addr >> 12) & 0x1FF
    }

    fn page_offset(addr: usize) -> usize {
        addr & 0xFFF
    }
}

/// Represents a single entry in a translation table
#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageEntry<L>(u64, PhantomData<L>);

impl PageEntry<L3> {
    /// Creates a 4KiB page mapping
    pub fn page(phys: usize, attrs: PageAttributes) -> Self {
        Self(
            (phys as u64)
                | (PageAttributes::PAGE | PageAttributes::PRESENT | PageAttributes::ACCESS | attrs)
                    .bits(),
            PhantomData,
        )
    }
}

impl PageEntry<L2> {
    /// Creates a 2MiB page mapping
    pub fn block(phys: usize, attrs: PageAttributes) -> Self {
        Self(
            (phys as u64)
                | (PageAttributes::BLOCK
                    | PageAttributes::PRESENT
                    | PageAttributes::ACCESS
                    | attrs)
                    .bits(),
            PhantomData,
        )
    }

    /// Creates a mapping pointing to the next-level translation table
    pub fn table(phys: usize, attrs: PageAttributes) -> Self {
        Self(
            (phys as u64) | (PageAttributes::TABLE | PageAttributes::PRESENT | attrs).bits(),
            PhantomData,
        )
    }
}

impl PageEntry<L1> {
    /// Creates a mapping pointing to the next-level translation table
    pub fn table(phys: usize, attrs: PageAttributes) -> Self {
        Self(
            (phys as u64) | (PageAttributes::TABLE | PageAttributes::PRESENT | attrs).bits(),
            PhantomData,
        )
    }
}

impl<L: EntryLevel> PageEntry<L> {
    /// Represents an absent/invalid mapping in the table
    pub const INVALID: Self = Self(0, PhantomData);

    /// Converts a raw mapping value into this wrapper type
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure that `raw` is a valid mapping value for the
    /// current translation level.
    pub unsafe fn from_raw(raw: u64) -> Self {
        Self(raw, PhantomData)
    }
}

impl NextPageTable for PageTable<L1> {
    type NextLevel = PageTable<L2>;

    fn get_mut(&mut self, _index: usize) -> Option<&mut Self::NextLevel> {
        todo!()
    }

    fn get_mut_or_alloc(&mut self, _index: usize) -> &mut Self::NextLevel {
        todo!()
    }
}

impl NextPageTable for PageTable<L2> {
    type NextLevel = PageTable<L3>;

    fn get_mut(&mut self, _index: usize) -> Option<&mut Self::NextLevel> {
        todo!()
    }

    fn get_mut_or_alloc(&mut self, _index: usize) -> &mut Self::NextLevel {
        todo!()
    }
}

impl<L: EntryLevel> PageTable<L> {
    /// Constructs a page table with all entries marked as invalid
    pub const fn zeroed() -> Self {
        Self {
            data: [PageEntry::INVALID; 512],
        }
    }

    /// Returns a physical address pointing to this page table
    pub fn physical_address(&self) -> usize {
        unsafe { (self.data.as_ptr() as usize).physicalize() }
    }
}

impl<L: EntryLevel> Index<usize> for PageTable<L> {
    type Output = PageEntry<L>;

    fn index(&self, index: usize) -> &Self::Output {
        &self.data[index]
    }
}

impl<L: EntryLevel> IndexMut<usize> for PageTable<L> {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.data[index]
    }
}

impl FixedTables {
    /// Constructs a fixed table struct with all tables set to invalid values
    pub const fn zeroed() -> Self {
        Self {
            l1: PageTable::zeroed(),
            l2: PageTable::zeroed(),
            kernel_l3: PageTable::zeroed(),
            device_l3: PageTable::zeroed(),

            l1i: L1::index(KERNEL_PHYS_BASE),

            kernel_l2i: L2::index(KERNEL_PHYS_BASE),

            device_l2i: L2::index(KERNEL_PHYS_BASE) + 1,
            device_l3i: 0,

            pages_l2i: L2::index(KERNEL_PHYS_BASE) + 2,
        }
    }

    /// Initializes the kernel fixed tables from `src_tables` and sets up the necessary mappings
    /// for device/page array management.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure the function has not yet been called and that
    /// `src_tables` points to a correct virtual address of the compile-time translation tables.
    pub unsafe fn init(&mut self, src_tables: *const KernelTables) {
        // Copy kernel mapping entries from the initial L3 table
        for i in 0..512 {
            KERNEL_TABLES.kernel_l3[i] = PageEntry::from_raw((*src_tables).l3.data[i]);
        }

        // Map L1 -> L2 -> L3 for kernel
        KERNEL_TABLES.l1[self.l1i] =
            PageEntry::<L1>::table(KERNEL_TABLES.l2.physical_address(), PageAttributes::empty());
        KERNEL_TABLES.l2[self.kernel_l2i] = PageEntry::<L2>::table(
            KERNEL_TABLES.kernel_l3.physical_address(),
            PageAttributes::empty(),
        );
        KERNEL_TABLES.l2[self.device_l2i] = PageEntry::<L2>::table(
            KERNEL_TABLES.device_l3.physical_address(),
            PageAttributes::empty(),
        );

        // Map physical page array
        let page_array_phys = (self.l1i << 30) | (self.pages_l2i << 21);
        KERNEL_TABLES.l2[self.pages_l2i] =
            PageEntry::<L2>::block(page_array_phys, PageAttributes::empty());
    }

    /// Returns the physical address of the upmost translation table
    pub fn l1_physical_address(&self) -> usize {
        self.l1.physical_address()
    }

    /// Returns the range to which the page tracking array is mapped
    pub fn page_array_range(&self) -> (usize, usize) {
        ((self.l1i << 30) | (self.pages_l2i << 21), 1 << 21)
    }

    /// Maps a single 4KiB page for device MMIO.
    ///
    /// # Safety
    ///
    /// The caller is responsible for making sure the `phys` address is valid and is not aliased.
    pub unsafe fn map_device_4k(&mut self, phys: usize) -> usize {
        if self.device_l3i == 512 {
            panic!("Ran out of device mapping memory");
        }

        let virt = (1 << 30) | (1 << 21) | (self.device_l3i << 12) | KERNEL_VIRT_OFFSET;
        self.device_l3[self.device_l3i] = PageEntry::page(phys, PageAttributes::empty());

        self.device_l3i += 1;

        virt
    }
}

/// Global kernel virtual memory tables
pub static mut KERNEL_TABLES: FixedTables = FixedTables::zeroed();
