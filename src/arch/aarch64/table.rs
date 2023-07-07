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

#[derive(Clone)]
#[repr(C, align(0x1000))]
pub struct AddressSpace {
    l1: PageTable<L1>,
}

#[derive(Clone)]
#[repr(C, align(0x1000))]
pub struct PageTable<L: EntryLevel> {
    data: [PageEntry<L>; 512],
}

pub struct FixedTables {
    l1: PageTable<L1>,
    l2: PageTable<L2>,
    kernel_l3: PageTable<L3>,
    device_l3: PageTable<L3>,

    l1i: usize,

    kernel_l2i: usize,

    device_l2i: usize,
    device_l3i: usize,
}

#[derive(Clone)]
pub struct L1;
#[derive(Clone)]
pub struct L2;
#[derive(Clone)]
pub struct L3;

bitflags! {
    pub struct PageAttributes: u64 {
        const PRESENT = 1 << 0;

        // For L1, L2
        const TABLE = 1 << 1;
        // For L3
        const PAGE = 1 << 1;
        // For L1, L2
        const BLOCK = 0 << 1;

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

#[derive(Clone, Copy)]
#[repr(transparent)]
pub struct PageEntry<L>(u64, PhantomData<L>);

impl PageEntry<L3> {
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
    pub fn table(phys: usize, attrs: PageAttributes) -> Self {
        Self(
            (phys as u64) | (PageAttributes::TABLE | PageAttributes::PRESENT | attrs).bits(),
            PhantomData,
        )
    }
}

impl PageEntry<L1> {
    pub fn table(phys: usize, attrs: PageAttributes) -> Self {
        Self(
            (phys as u64) | (PageAttributes::TABLE | PageAttributes::PRESENT | attrs).bits(),
            PhantomData,
        )
    }
}

impl<L: EntryLevel> PageEntry<L> {
    pub const INVALID: Self = Self(0, PhantomData);

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
    pub const fn zeroed() -> Self {
        Self {
            data: [PageEntry::INVALID; 512],
        }
    }

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
        }
    }

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
    }

    pub fn l1_physical_address(&self) -> usize {
        self.l1.physical_address()
    }

    pub fn map_4k(&mut self, phys: usize) -> usize {
        if self.device_l3i == 512 {
            panic!("Ran out of device mapping memory");
        }

        let virt = (1 << 30) | (1 << 21) | (self.device_l3i << 12) | KERNEL_VIRT_OFFSET;
        self.device_l3[self.device_l3i] = PageEntry::page(phys, PageAttributes::empty());

        self.device_l3i += 1;

        virt
    }
}

pub static mut KERNEL_TABLES: FixedTables = FixedTables::zeroed();
