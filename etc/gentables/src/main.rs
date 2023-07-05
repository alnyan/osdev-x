use std::{
    fs::{File, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    mem::size_of,
};

use bitflags::bitflags;
use bytemuck::bytes_of;
use clap::Parser;
use elf::{
    abi::{PF_W, PF_X, PT_LOAD},
    endian::AnyEndian,
    ElfStream,
};
use tables::{KernelTables, RawTable};

type Elf<S> = ElfStream<AnyEndian, S>;

bitflags! {
    struct PageAttributes: u64 {
        const PRESENT = 1 << 0;
        const TABLE = 1 << 1;

        // Lower attributes
        const NOT_GLOBAL = 1 << 11;
        const AF = 1 << 10;
        const BLOCK_ISH = 3 << 8;

        const AP_READWRITE = 0 << 6;
        const AP_READONLY = 2 << 6;

        // Upper attributes
        const UXN_TABLE = 1 << 60;
        const UXN = 1 << 54;
        const PXN = 1 << 53;
    }
}

#[derive(Parser)]
struct Args {
    path: String,
}

fn extract_tables_section<S: Read + Seek>(elf: &mut Elf<S>) -> (u64, u64) {
    let (shdrs, strtab) = elf.section_headers_with_strtab().unwrap();
    let strtab = strtab.unwrap();

    for shdr in shdrs {
        let name = strtab.get(shdr.sh_name as usize).unwrap();

        if name == ".data.tables" {
            assert_eq!(shdr.sh_size, size_of::<KernelTables>() as u64);
            assert_eq!(shdr.sh_addr & 0xFFF, 0);

            return (shdr.sh_offset, shdr.sh_addr);
        }
    }

    panic!();
}

fn map_page(l3: &mut RawTable, phys: u64, elf_attrs: u32) {
    let l3i = (phys >> 12) & 0x1FF;

    let rw = if elf_attrs & PF_W != 0 {
        PageAttributes::AP_READWRITE
    } else {
        PageAttributes::AP_READONLY
    };
    let x = if elf_attrs & PF_X != 0 {
        PageAttributes::empty()
    } else {
        PageAttributes::PXN
    };

    l3.data[l3i as usize] = (phys & !0xFFF)
        | (rw
            | x
            | PageAttributes::UXN
            | PageAttributes::TABLE
            | PageAttributes::AF
            | PageAttributes::PRESENT)
            .bits();
}

fn find_kernel_range<S: Read + Seek>(elf: &Elf<S>) -> (u64, u64) {
    let mut kernel_start = u64::MAX;
    let mut kernel_end = u64::MIN;

    for phdr in elf.segments() {
        if phdr.p_type == PT_LOAD {
            let start_page = phdr.p_paddr & !0xFFF;
            let end_page = (phdr.p_paddr + phdr.p_memsz + 0xFFF) & !0xFFF;

            if start_page < kernel_start {
                kernel_start = start_page;
            }
            if end_page > kernel_end {
                kernel_end = end_page;
            }
        }
    }

    (kernel_start, kernel_end)
}

fn generate_tables<S: Read + Seek>(file: S) -> (KernelTables, u64) {
    let mut elf: Elf<_> = ElfStream::open_stream(file).unwrap();

    let (tables_offset, tables_paddr) = extract_tables_section(&mut elf);

    let (kernel_start, kernel_end) = find_kernel_range(&elf);

    let kernel_start_2m = kernel_start & !0x1FFFFF;
    let kernel_end_2m = (kernel_end + 0x1FFFFF) & !0x1FFFFF;

    if kernel_end_2m - kernel_start_2m > (1 << 21) {
        panic!();
    }

    for phdr in elf.segments() {
        if phdr.p_type != PT_LOAD {
            continue;
        }
    }

    assert_eq!(kernel_end >> 39, 0);

    let mut tables = KernelTables::zeroed();

    let l1i = (kernel_start >> 30) & 0x1FF;
    let l2i = (kernel_start >> 21) & 0x1FF;
    let l2_table_base = tables_paddr + 0x1000;
    let l3_table_base = tables_paddr + 0x3000;

    // Map 1G in which the kernel is placed to L2 table
    tables.l1.data[l1i as usize] =
        l2_table_base | (PageAttributes::PRESENT | PageAttributes::TABLE).bits();

    // Map 2M in which the kernel is placed to L3 table
    tables.l2[0].data[l2i as usize] =
        l3_table_base | (PageAttributes::PRESENT | PageAttributes::TABLE).bits();

    // Map the kernel pages
    for phdr in elf.segments() {
        if phdr.p_type != PT_LOAD {
            continue;
        }

        let segment_start = phdr.p_paddr & !0xFFF;
        let segment_end = (phdr.p_paddr + phdr.p_memsz + 0xFFF) & !0xFFF;

        println!(
            "{:#x}..{:#x}: r{}{}",
            segment_start,
            segment_end,
            if phdr.p_flags & PF_W != 0 { "w" } else { "-" },
            if phdr.p_flags & PF_X != 0 { "x" } else { "-" }
        );

        for page in (segment_start..segment_end).step_by(0x1000) {
            map_page(&mut tables.l3[0], page, phdr.p_flags);
        }
    }

    (tables, tables_offset)
}

fn write_tables<S: Write + Seek>(mut file: S, tables: &KernelTables, tables_offset: u64) {
    file.seek(SeekFrom::Start(tables_offset)).unwrap();
    let table_bytes = bytes_of(tables);
    file.write_all(table_bytes).unwrap();
}

fn main() {
    let args = Args::parse();

    let (tables, tables_offset) = {
        let src_file = File::open(&args.path).unwrap();

        generate_tables(src_file)
    };

    let dst_file = OpenOptions::new()
        .write(true)
        .truncate(false)
        .open(&args.path)
        .unwrap();
    write_tables(dst_file, &tables, tables_offset);
}
