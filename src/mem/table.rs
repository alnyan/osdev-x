use core::{
    marker::PhantomData,
    ops::{Index, IndexMut},
};

use bitflags::bitflags;

use super::ConvertAddress;

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

pub trait NextPageTable {
    type NextLevel: EntryLevel;

    fn get_mut_or_alloc(&mut self, index: usize) -> &mut PageTable<Self::NextLevel>;
    fn get_mut(&mut self, index: usize) -> Option<&mut PageTable<Self::NextLevel>>;
}

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

#[const_trait]
pub trait EntryLevel: Clone {
    fn index(addr: usize) -> usize;
    fn page_offset(addr: usize) -> usize;
}
#[derive(Clone)]
pub struct L1;
#[derive(Clone)]
pub struct L2;
#[derive(Clone)]
pub struct L3;

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
    type NextLevel = L2;

    fn get_mut(&mut self, _index: usize) -> Option<&mut PageTable<Self::NextLevel>> {
        todo!()
    }

    fn get_mut_or_alloc(&mut self, _index: usize) -> &mut PageTable<Self::NextLevel> {
        todo!()
    }
}

impl NextPageTable for PageTable<L2> {
    type NextLevel = L3;

    fn get_mut(&mut self, _index: usize) -> Option<&mut PageTable<Self::NextLevel>> {
        todo!()
    }

    fn get_mut_or_alloc(&mut self, _index: usize) -> &mut PageTable<Self::NextLevel> {
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
