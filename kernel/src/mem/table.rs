//! Virtual memory table interface
use abi::error::Error;

pub use crate::arch::aarch64::table::{AddressSpace, PageAttributes, PageEntry, PageTable};

/// Interface for virtual memory address space management
pub trait VirtualMemoryManager {
    /// Allocates a region of virtual memory inside the address space and maps it to physical
    /// memory pages with given attributes
    fn allocate(
        &self,
        hint: Option<usize>,
        len: usize,
        attrs: PageAttributes,
    ) -> Result<usize, Error>;

    /// Releases the virtual memory region from the address space and the pages it refers to
    fn deallocate(&self, addr: usize, len: usize) -> Result<(), Error>;
}

/// Interface for non-terminal tables to retrieve the next level of address translation tables
pub trait NextPageTable {
    /// Type for the next-level page table
    type NextLevel;

    /// Tries looking up a next-level table at given index, allocating and mapping one if it is not
    /// present there
    fn get_mut_or_alloc(&mut self, index: usize) -> Result<&'static mut Self::NextLevel, Error>;
    /// Returns a mutable reference to a next-level table at `index`, if present
    fn get_mut(&mut self, index: usize) -> Option<&'static mut Self::NextLevel>;
}

/// Interface for a single level of address translation
#[const_trait]
pub trait EntryLevel: Copy {
    /// Returns the index into a page table for a given address
    fn index(addr: usize) -> usize;
    /// Returns the offset of an address from the page start at current level
    fn page_offset(addr: usize) -> usize;
}
