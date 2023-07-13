//! System function call handlers
use abi::{error::Error, SyscallFunction};

fn arg_buffer_ref<'a>(base: usize, len: usize) -> Result<&'a [u8], Error> {
    if base + len > crate::mem::KERNEL_VIRT_OFFSET {
        panic!("Invalid argument");
    }
    Ok(unsafe { core::slice::from_raw_parts(base as *const u8, len) })
}

fn arg_user_str<'a>(base: usize, len: usize) -> Result<&'a str, Error> {
    let slice = arg_buffer_ref(base, len)?;
    Ok(core::str::from_utf8(slice).unwrap())
}

/// Entrypoint for system calls that takes raw argument values
pub fn raw_syscall_handler(func: u64, args: &[u64]) -> u64 {
    let Some(func) = SyscallFunction::from_repr(func as usize) else {
        todo!("Undefined syscall: {}", func);
    };

    match func {
        SyscallFunction::DoSomething => {
            let arg = arg_user_str(args[0] as usize, args[1] as usize).unwrap();
            debugln!("User string: {:?}", arg);
            0
            // 0
        }
    }
}
