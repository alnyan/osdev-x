//! AArch64 virtual memory management facilities
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;

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

pub struct FixedTables {
    l1: PageTable<L1>,
    device_l2: PageTable<L2>,
    device_l3: PageTable<L3>,

    device_l2i: usize,
    device_l3i: usize,
}

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
        // &self may already by a physical address
        let addr = self.data.as_ptr() as usize;
        if addr < KERNEL_VIRT_OFFSET {
            addr
        } else {
            unsafe { addr.physicalize() }
        }
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
    pub const fn zeroed() -> Self {
        Self {
            l1: PageTable::zeroed(),
            device_l2: PageTable::zeroed(),
            device_l3: PageTable::zeroed(),

            device_l2i: 1, // First entry is reserved for 4K table
            device_l3i: 0,
        }
    }

    pub fn map_device_pages(&mut self, phys: usize, count: usize) -> usize {
        if count > 512 {
            panic!("Unsupported device memory mapping size");
        } else if count > 1 {
            // 2MiB mappings
            todo!();
        } else {
            // 4KiB mappings
            if self.device_l3i == 512 {
                panic!("Out of device memory");
            }

            let virt = DEVICE_VIRT_OFFSET + (self.device_l3i << 12);
            self.device_l3[self.device_l3i] = PageEntry::page(phys, PageAttributes::empty());
            self.device_l3i += 1;

            virt
        }
    }
}

pub unsafe fn init_fixed_tables() {
    // Map first 256GiB
    for i in 0..256 {
        KERNEL_TABLES.l1[i] = PageEntry::<L1>::block(i << 30, PageAttributes::empty());
    }

    KERNEL_TABLES.l1[256] = PageEntry::<L1>::table(
        KERNEL_TABLES.device_l2.physical_address(),
        PageAttributes::empty(),
    );
    KERNEL_TABLES.device_l2[0] = PageEntry::<L2>::table(
        KERNEL_TABLES.device_l3.physical_address(),
        PageAttributes::empty(),
    );
}

pub const DEVICE_VIRT_OFFSET: usize = KERNEL_VIRT_OFFSET + (256 << 30);
pub static mut KERNEL_TABLES: FixedTables = FixedTables::zeroed();
