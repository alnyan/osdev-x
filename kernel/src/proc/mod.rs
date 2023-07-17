//! Internal management for processes

pub mod exec;
pub mod wait;

use aarch64_cpu::registers::TTBR0_EL1;
use elf::{
    abi::{PF_W, PF_X, PT_LOAD},
    endian::AnyEndian,
    ElfBytes,
};
use tock_registers::interfaces::Writeable;

use crate::{
    arch::aarch64::table::tlb_flush_vaae1,
    mem::{
        phys::{self, PageUsage},
        table::{AddressSpace, PageAttributes},
    },
};

fn load_segment(space: &mut AddressSpace, addr: usize, data: &[u8], memsz: usize, elf_attrs: u32) {
    let attrs = match (elf_attrs & PF_W, elf_attrs & PF_X) {
        (0, 0) => PageAttributes::AP_BOTH_READONLY,
        (_, 0) => PageAttributes::AP_BOTH_READWRITE,
        (0, _) => PageAttributes::AP_BOTH_READONLY,
        (_, _) => PageAttributes::AP_BOTH_READWRITE,
    };

    let aligned_start = addr & !0xFFF;
    let aligned_end = (addr + memsz + 0xFFF) & !0xFFF;

    // Map and write pages
    for page in (aligned_start..aligned_end).step_by(0x1000) {
        if let Some(_phys) = space.translate(page) {
            todo!();
        } else {
            let phys = phys::alloc_page(PageUsage::Used).unwrap();
            space
                .map_page(page, phys, PageAttributes::AP_BOTH_READWRITE)
                .unwrap();

            debugln!("MAP (alloc) {:#x} -> {:#x}", page, phys);
            tlb_flush_vaae1(page);
        }
    }

    unsafe {
        // Write the data
        let dst = core::slice::from_raw_parts_mut(addr as *mut u8, memsz);
        dst[..data.len()].copy_from_slice(data);

        // Zero the rest
        dst[data.len()..memsz].fill(0);
    }

    // Map the region as readonly
    for page in (aligned_start..aligned_end).step_by(0x1000) {
        let phys = space.translate(page).unwrap();
        space.map_page(page, phys, attrs).unwrap();
    }
}

/// Loads an ELF image into the address space from a slice
pub fn load_elf_from_memory(space: &mut AddressSpace, src: &[u8]) -> usize {
    // Map the address space temporarily
    TTBR0_EL1.set(space.physical_address() as u64);

    let elf = ElfBytes::<AnyEndian>::minimal_parse(src).unwrap();

    for phdr in elf.segments().unwrap() {
        if phdr.p_type != PT_LOAD {
            continue;
        }

        debugln!("LOAD {:#x}", phdr.p_vaddr);
        let data = &src[phdr.p_offset as usize..(phdr.p_offset + phdr.p_filesz) as usize];
        load_segment(
            space,
            phdr.p_vaddr as usize,
            data,
            phdr.p_memsz as usize,
            phdr.p_flags,
        );
    }

    TTBR0_EL1.set_baddr(0);

    elf.ehdr.e_entry as usize
}
