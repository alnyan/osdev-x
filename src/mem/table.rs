pub use crate::arch::aarch64::table::{
    AddressSpace, FixedTables, PageAttributes, PageEntry, PageTable, KERNEL_TABLES,
};

pub trait NextPageTable {
    type NextLevel;

    fn get_mut_or_alloc(&mut self, index: usize) -> &mut Self::NextLevel;
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::NextLevel>;
}

#[const_trait]
pub trait EntryLevel: Clone {
    fn index(addr: usize) -> usize;
    fn page_offset(addr: usize) -> usize;
}
