use abi::{SyscallArgument, SyscallFunction};

fn sys_do_something() {}

pub fn raw_syscall_handler(func: u64, args: &[u64]) -> u64 {
    let Some(func) = SyscallFunction::from_repr(func as usize) else {
        todo!("Undefined syscall");
    };

    match func {
        SyscallFunction::DoSomething => {
            let x = usize::from_syscall_argument(args[0] as usize).unwrap();
            let y = usize::from_syscall_argument(args[1] as usize).unwrap();

            (x + y) as u64
        }
    }
}
