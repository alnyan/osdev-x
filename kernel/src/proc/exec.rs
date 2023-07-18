//! Binary execution functions
use core::mem::size_of;

use abi::error::Error;
use alloc::rc::Rc;

use crate::{
    arch::aarch64::context::TaskContext,
    mem::{
        phys::{self, PageUsage},
        table::{AddressSpace, PageAttributes},
        ConvertAddress,
    },
    proc,
    task::process::Process,
};

fn setup_args(space: &mut AddressSpace, virt: usize, args: &[&str]) -> Result<(), Error> {
    // arg data len
    let args_size: usize = args.iter().map(|x| x.len()).sum();
    // 1 + arg ptr:len count
    let args_ptr_size = (1 + args.len() * 2) * size_of::<usize>();

    let total_size = args_size + args_ptr_size;

    if total_size > 0x1000 {
        todo!();
    }

    debugln!("arg data size = {}", args_size);

    let phys_page = phys::alloc_page(PageUsage::Used)?;
    // TODO check if this doesn't overwrite anything
    space.map_page(virt, phys_page, PageAttributes::AP_BOTH_READWRITE)?;

    let write = unsafe { phys_page.virtualize() };

    let mut offset = args_ptr_size;

    unsafe {
        (write as *mut usize).write_volatile(args.len());
    }

    for i in 0..args.len() {
        // Place the argument pointer
        let ptr_place = write + (i * 2 + 1) * size_of::<usize>();
        let len_place = ptr_place + size_of::<usize>();
        unsafe {
            (ptr_place as *mut usize).write_volatile(virt + offset);
            (len_place as *mut usize).write_volatile(args[i].len());
        }
        offset += args[i].len();
    }

    // Place the argument data
    unsafe {
        let arg_data_slice =
            core::slice::from_raw_parts_mut((write + args_ptr_size) as *mut u8, args_size);
        let mut offset = 0;
        for &s in args {
            arg_data_slice[offset..offset + s.len()].copy_from_slice(s.as_bytes());
            offset += s.len();
        }
    }

    Ok(())
}

/// Sets up a userspace structure from a slice defining an ELF binary
pub fn create_from_memory(data: &[u8], args: &[&str]) -> Result<Rc<Process>, Error> {
    const USER_STACK_PAGES: usize = 8;

    let mut space = AddressSpace::new_empty()?;
    let elf_entry = proc::load_elf_from_memory(&mut space, data);

    let virt_stack_base = 0x10000000;
    // 0x1000 of guard page
    let virt_args_base = virt_stack_base + (USER_STACK_PAGES + 1) * 0x1000;

    for i in 0..USER_STACK_PAGES {
        let phys = phys::alloc_page(PageUsage::Used)?;
        space.map_page(
            virt_stack_base + i * 0x1000,
            phys,
            PageAttributes::AP_BOTH_READWRITE,
        )?;
    }

    setup_args(&mut space, virt_args_base, args)?;

    debugln!("Entry: {:#x}", elf_entry);

    let context = TaskContext::user(
        elf_entry,
        virt_args_base,
        space.physical_address(),
        virt_stack_base + USER_STACK_PAGES * 0x1000,
    )?;

    Ok(Process::new_with_context(Some(space), context))
}
