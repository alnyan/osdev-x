use abi::SyscallFunction;

fn sys_do_something() {}

pub fn raw_syscall_handler(func: u64, args: &[u64]) -> u64 {
    let Some(func) = SyscallFunction::from_repr(func as usize) else {
        todo!("Undefined syscall");
    };

    match func {
        SyscallFunction::DoSomething => {
            todo!();
        }
    }
}
