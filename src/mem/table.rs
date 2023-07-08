//! Virtual memory table interface
pub use crate::arch::aarch64::table::{
    AddressSpace, FixedTables, PageAttributes, PageEntry, PageTable, KERNEL_TABLES,
};

/// Interface for non-terminal tables to retrieve the next level of address translation tables
pub trait NextPageTable {
    /// Type for the next-level page table
    type NextLevel;

    /// Tries looking up a next-level table at given index, allocating and mapping one if it is not
    /// present there
    fn get_mut_or_alloc(&mut self, index: usize) -> &mut Self::NextLevel;
    /// Returns a mutable reference to a next-level table at `index`, if present
    fn get_mut(&mut self, index: usize) -> Option<&mut Self::NextLevel>;
}

/// Interface for a single level of address translation
#[const_trait]
pub trait EntryLevel: Clone {
    /// Returns the index into a page table for a given address
    fn index(addr: usize) -> usize;
    /// Returns the offset of an address from the page start at current level
    fn page_offset(addr: usize) -> usize;
}
