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
    // 1G
    pub l1: RawTable,
    // 2M
    pub l2: [RawTable; 2],
    // 4K
    pub l3: [RawTable; 4],
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
            l2: [RawTable::zeroed(); 2],
            l3: [RawTable::zeroed(); 4],
        }
    }
}
