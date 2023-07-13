//! System call handling functions
use abi::SyscallFunction;

/// Entry for system call handling that accepts raw register values
pub fn raw_syscall_handler(func: u64, _args: &[u64]) -> u64 {
    let Some(func) = SyscallFunction::from_repr(func as usize) else {
        todo!("Undefined syscall");
    };

    match func {
        SyscallFunction::DoSomething => {
            todo!();
        }
    }
}
