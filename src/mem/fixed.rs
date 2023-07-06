use tables::{KernelTables, RawTable};

use super::{device::DeviceMemoryIo, KERNEL_VIRT_OFFSET};

pub struct FixedTables {
    l1: RawTable,
    l2: RawTable,
    l3: [RawTable; 2],

    l3_page_index: usize,
}

impl FixedTables {
    pub const fn zeroed() -> Self {
        Self {
            l1: RawTable::zeroed(),
            l2: RawTable::zeroed(),
            l3: [RawTable::zeroed(); 2],

            l3_page_index: 0,
        }
    }

    pub unsafe fn init(&mut self, src_tables: *const KernelTables) {
        KERNEL_TABLES.l3[0] = (*src_tables).l3;

        // Map L1 -> L2 -> L3 for kernel
        let l2_addr = (&KERNEL_TABLES.l2 as *const _ as usize - KERNEL_VIRT_OFFSET) as u64;
        KERNEL_TABLES.l1.data[1] = l2_addr | (1 << 0 | 1 << 1);
        let l3_addr = ((&KERNEL_TABLES.l3[0] as *const _ as usize) - KERNEL_VIRT_OFFSET) as u64;
        KERNEL_TABLES.l2.data[0] = l3_addr | (1 << 0 | 1 << 1);

        // Setup a 2M region for MMIO
        let l3_addr = ((&KERNEL_TABLES.l3[1] as *const _ as usize) - KERNEL_VIRT_OFFSET) as u64;
        KERNEL_TABLES.l2.data[1] = l3_addr | (1 << 0 | 1 << 1);
    }

    pub fn l1_ptr(&self) -> *const RawTable {
        self.l1.data.as_ptr() as _
    }

    pub fn map_4k(&mut self, phys: usize) -> usize {
        if self.l3_page_index == 512 {
            loop {}
        }

        let virt = (1 << 30) | (1 << 21) | (self.l3_page_index << 12) | KERNEL_VIRT_OFFSET;
        self.l3[1].data[self.l3_page_index] = (phys as u64) | (1 << 0 | 1 << 1 | 1 << 10);

        self.l3_page_index += 1;

        virt
    }
}

pub(super) static mut KERNEL_TABLES: FixedTables = FixedTables::zeroed();
