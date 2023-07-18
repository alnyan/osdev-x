//! System function call handlers
use core::time::Duration;

use abi::{
    error::{Error, IntoSyscallResult},
    io::{OpenFlags, RawFd},
    SyscallFunction,
};
use vfs::{Read, Write};

use crate::{
    mem::table::{PageAttributes, VirtualMemoryManager},
    proc::wait,
    task::process::Process,
};

fn arg_buffer_ref<'a>(base: usize, len: usize) -> Result<&'a [u8], Error> {
    if base + len > crate::mem::KERNEL_VIRT_OFFSET {
        panic!("Invalid argument");
    }
    Ok(unsafe { core::slice::from_raw_parts(base as *const u8, len) })
}

fn arg_buffer_mut<'a>(base: usize, len: usize) -> Result<&'a mut [u8], Error> {
    if base + len > crate::mem::KERNEL_VIRT_OFFSET {
        panic!("Invalid argument");
    }
    Ok(unsafe { core::slice::from_raw_parts_mut(base as *mut u8, len) })
}

fn arg_user_str<'a>(base: usize, len: usize) -> Result<&'a str, Error> {
    let slice = arg_buffer_ref(base, len)?;
    Ok(core::str::from_utf8(slice).unwrap())
}

/// Entrypoint for system calls that takes raw argument values
pub fn raw_syscall_handler(func: u64, args: &[u64]) -> u64 {
    let Ok(func) = SyscallFunction::try_from(func as usize) else {
        todo!("Undefined syscall: {}", func);
    };

    match func {
        SyscallFunction::DebugTrace => {
            let pid = Process::get_current()
                .as_deref()
                .map(Process::id)
                .unwrap_or(0);
            let arg = arg_user_str(args[0] as usize, args[1] as usize).unwrap();
            debugln!("[{}] TRACE: {:?}", pid, arg);
            0
            // 0
        }
        SyscallFunction::Nanosleep => {
            let seconds = args[0];
            let nanos = args[1] as u32;
            let duration = Duration::new(seconds, nanos);
            let mut remaining = Duration::ZERO;

            wait::sleep(duration, &mut remaining).unwrap();

            0
        }
        SyscallFunction::Exit => {
            Process::current().exit(args[0] as _);
            panic!();
        }
        SyscallFunction::MapMemory => {
            let len = args[1] as usize;

            let proc = Process::current();
            let space = proc.address_space();

            if len & 0xFFF != 0 {
                todo!();
            }

            let addr = space.allocate(None, len / 0x1000, PageAttributes::AP_BOTH_READWRITE);
            debugln!("mmap({:#x}) = {:x?}", len, addr);

            addr.into_syscall_result() as u64
        }
        SyscallFunction::UnmapMemory => {
            let addr = args[0] as usize;
            let len = args[1] as usize;

            let proc = Process::current();
            let space = proc.address_space();

            if len & 0xFFF != 0 {
                todo!();
            }

            let res = space.deallocate(addr, len);
            debugln!("munmap({:#x}, {:#x})", addr, len);

            res.into_syscall_result() as u64
        }
        SyscallFunction::Write => {
            let fd = RawFd(args[0] as u32);
            let data = arg_buffer_ref(args[1] as _, args[2] as _).unwrap();

            let proc = Process::current();
            let io = proc.io.lock();
            let file = io.file(fd).unwrap();
            let mut file_borrow = file.borrow_mut();

            file_borrow.write(data).into_syscall_result() as u64
        }
        SyscallFunction::Read => {
            let fd = RawFd(args[0] as u32);
            let data = arg_buffer_mut(args[1] as _, args[2] as _).unwrap();

            let proc = Process::current();
            let io = proc.io.lock();
            let file = io.file(fd).unwrap();
            let mut file_borrow = file.borrow_mut();

            file_borrow.read(data).into_syscall_result() as u64
        }
        SyscallFunction::Open => {
            let path = arg_user_str(args[0] as usize, args[1] as usize).unwrap();
            let opts = OpenFlags(args[2] as u32);

            debugln!("sys_open {}", path);

            let proc = Process::current();
            let mut io = proc.io.lock();

            let file = io.ioctx().open(None, path, opts).unwrap();
            let fd = io.place_file(file);

            fd.into_syscall_result() as u64
        }
        SyscallFunction::Close => {
            let fd = RawFd(args[0] as u32);

            let proc = Process::current();
            let mut io = proc.io.lock();
            io.close_file(fd).into_syscall_result() as u64
        }
    }
}
