#![no_std]

#[cfg(feature = "bytemuck")]
use bytemuck::{Pod, Zeroable};

#[derive(Clone, Copy)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C, align(0x1000))]
pub struct RawTable {
    pub data: [u64; 512],
}

#[derive(Clone, Copy)]
#[cfg_attr(feature = "bytemuck", derive(Pod, Zeroable))]
#[repr(C, align(0x1000))]
pub struct KernelTables {
    // 1G tables
    pub l1: RawTable,
    // 2M tables
    pub l2: RawTable,
    // 4K pages
    pub l3: RawTable,
}

impl RawTable {
    pub const fn zeroed() -> Self {
        Self { data: [0; 512] }
    }
}

impl KernelTables {
    pub const fn zeroed() -> Self {
        Self {
            l1: RawTable::zeroed(),
            l2: RawTable::zeroed(),
            l3: RawTable::zeroed(),
        }
    }
}
