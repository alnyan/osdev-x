//! AArch64 virtual memory management facilities
use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use abi::error::Error;
use bitflags::bitflags;

use crate::mem::{
    phys::{self, PageUsage},
    table::{EntryLevel, NextPageTable},
    ConvertAddress, KERNEL_VIRT_OFFSET,
};

/// TODO
#[derive(Clone)]
#[repr(C, align(0x1000))]
pub struct AddressSpace {
    l1: *mut PageTable<L1>,
}

/// Page table representing a single level of address translation
#[derive(Clone)]
#[repr(C, align(0x1000))]
pub struct PageTable<L: EntryLevel> {
    data: [PageEntry<L>; 512],
}

/// Translation level 1: Entry is 1GiB page/table
#[derive(Clone, Copy)]
pub struct L1;
/// Translation level 2: Entry is 2MiB page/table
#[derive(Clone, Copy)]
pub struct L2;
/// Translation level 3: Entry is 4KiB page
#[derive(Clone, Copy)]
pub struct L3;

/// Tag trait to mark that the page table level may point to a next-level table
pub trait NonTerminalEntryLevel: EntryLevel {
    /// Tag type of the level this entry level may point to
    type NextLevel: EntryLevel;
}

impl NonTerminalEntryLevel for L1 {
    type NextLevel = L2;
}
impl NonTerminalEntryLevel for L2 {
    type NextLevel = L3;
}

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

        /// For page/block mappings, allows both user and kernel code to read/write to the page
        const AP_BOTH_READWRITE = 1 << 6;
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

/// Fixed-layout kernel-space address mapping tables
pub struct FixedTables {
    l1: PageTable<L1>,
    device_l2: PageTable<L2>,
    device_l3: PageTable<L3>,

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

impl<T: NonTerminalEntryLevel> PageEntry<T> {
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

    /// Returns the physical address of the table this entry refers to, returning None if it
    /// does not
    pub fn as_table(self) -> Option<usize> {
        if self.0 & (PageAttributes::TABLE | PageAttributes::PRESENT).bits()
            == (PageAttributes::TABLE | PageAttributes::PRESENT).bits()
        {
            Some((self.0 & !0xFFF) as usize)
        } else {
            None
        }
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

impl<L: NonTerminalEntryLevel> NextPageTable for PageTable<L> {
    type NextLevel = PageTable<L::NextLevel>;

    fn get_mut(&mut self, index: usize) -> Option<&'static mut Self::NextLevel> {
        let entry = self[index];

        entry
            .as_table()
            .map(|addr| unsafe { &mut *(addr.virtualize() as *mut Self::NextLevel) })
    }

    fn get_mut_or_alloc(&mut self, index: usize) -> Result<&'static mut Self::NextLevel, Error> {
        let entry = self[index];

        if let Some(table) = entry.as_table() {
            Ok(unsafe { &mut *(table.virtualize() as *mut Self::NextLevel) })
        } else {
            let table = PageTable::new_zeroed()?;
            self[index] = PageEntry::<L>::table(table.physical_address(), PageAttributes::empty());
            Ok(table)
        }
    }
}

impl<L: EntryLevel> PageTable<L> {
    /// Constructs a page table with all entries marked as invalid
    pub const fn zeroed() -> Self {
        Self {
            data: [PageEntry::INVALID; 512],
        }
    }

    /// Allocates a new page table, filling it with non-preset entries
    pub fn new_zeroed() -> Result<&'static mut Self, Error> {
        let page = unsafe { phys::alloc_page(PageUsage::Used)?.virtualize() };
        let table = unsafe { &mut *(page as *mut Self) };
        for i in 0..512 {
            table[i] = PageEntry::INVALID;
        }
        Ok(table)
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
    /// Constructs an empty table group
    pub const fn zeroed() -> Self {
        Self {
            l1: PageTable::zeroed(),
            device_l2: PageTable::zeroed(),
            device_l3: PageTable::zeroed(),

            device_l3i: 0,
        }
    }

    /// Maps a physical memory region as device memory and returns its allocated base address
    pub fn map_device_pages(&mut self, phys: usize, count: usize) -> Result<usize, Error> {
        if count > 512 * 512 {
            panic!("Unsupported device memory mapping size");
        } else if count > 512 {
            // 2MiB mappings
            todo!();
        } else {
            // 4KiB mappings
            if self.device_l3i + count > 512 {
                return Err(Error::OutOfMemory);
            }

            let virt = DEVICE_VIRT_OFFSET + (self.device_l3i << 12);
            for i in 0..count {
                self.device_l3[self.device_l3i + i] =
                    PageEntry::page(phys + i * 0x1000, PageAttributes::empty());
            }
            self.device_l3i += count;

            tlb_flush_vaae1(virt);

            Ok(virt)
        }
    }
}

impl AddressSpace {
    /// Allocates an empty address space with all entries marked as non-present
    pub fn empty() -> Result<Self, Error> {
        let l1 = unsafe { phys::alloc_page(PageUsage::Used)?.virtualize() as *mut PageTable<L1> };

        for i in 0..512 {
            unsafe {
                (*l1)[i] = PageEntry::INVALID;
            }
        }

        Ok(Self { l1 })
    }

    unsafe fn as_mut(&self) -> &'static mut PageTable<L1> {
        self.l1.as_mut().unwrap()
    }

    /// Inserts a single 4KiB virt -> phys mapping into the address apce
    pub fn map_page(&self, virt: usize, phys: usize, attrs: PageAttributes) -> Result<(), Error> {
        let l1i = L1::index(virt);
        let l2i = L2::index(virt);
        let l3i = L3::index(virt);

        let l2 = unsafe { self.as_mut().get_mut_or_alloc(l1i) }?;
        let l3 = l2.get_mut_or_alloc(l2i)?;

        debugln!(
            "[{:#x}] map {:#x} -> {:#x}",
            self.physical_address(),
            virt,
            phys
        );

        l3[l3i] = PageEntry::page(phys, attrs);

        Ok(())
    }

    /// Returns the physical address of the address space (to be used in a TTBRn_ELx)
    pub fn physical_address(&self) -> usize {
        unsafe { (self.l1 as usize).physicalize() }
    }
}

fn tlb_flush_vaae1(page: usize) {
    unsafe {
        core::arch::asm!("tlbi vaae1, {addr}", addr = in(reg) page);
    }
}

/// Initializes mappings for the kernel and device memory tables.
///
/// # Safety
///
/// Only allowed to be called once during lower-half part of the initialization process.
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

/// Offset applied to device virtual memory mappings
pub const DEVICE_VIRT_OFFSET: usize = KERNEL_VIRT_OFFSET + (256 << 30);
/// Global kernel address space translation tables
pub static mut KERNEL_TABLES: FixedTables = FixedTables::zeroed();
