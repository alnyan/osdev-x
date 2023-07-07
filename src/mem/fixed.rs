use tables::KernelTables;

use super::{
    table::{PageAttributes, PageEntry, PageTable, L1, L2, L3},
    VirtualAddressParts, KERNEL_PHYS_BASE, KERNEL_VIRT_OFFSET,
};

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

impl FixedTables {
    pub const fn zeroed() -> Self {
        Self {
            l1: PageTable::zeroed(),
            l2: PageTable::zeroed(),
            kernel_l3: PageTable::zeroed(),
            device_l3: PageTable::zeroed(),

            l1i: KERNEL_PHYS_BASE.l1i(),

            kernel_l2i: KERNEL_PHYS_BASE.l2i(),

            device_l2i: KERNEL_PHYS_BASE.l2i() + 1,
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

pub(super) static mut KERNEL_TABLES: FixedTables = FixedTables::zeroed();
