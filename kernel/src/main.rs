//! osdev-x kernel crate
#![feature(
    naked_functions,
    asm_const,
    panic_info_message,
    optimize_attribute,
    const_trait_impl,
    maybe_uninit_slice,
    linked_list_cursors
)]
#![allow(clippy::new_without_default)]
#![warn(missing_docs)]
#![no_std]
#![no_main]

use abi::io::{OpenFlags, RawFd};
use task::process::Process;
use vfs::IoContext;

use crate::fs::devfs;

extern crate alloc;

#[macro_use]
pub mod debug;
#[macro_use]
pub mod arch;

pub mod device;
pub mod fs;
pub mod mem;
pub mod panic;
pub mod proc;
pub mod sync;
pub mod syscall;
pub mod task;
pub mod util;

/// Entry point for common kernel code.
///
/// # Note
///
/// This function is meant to be used as a kernel-space process after all the platform-specific
/// initialization has finished.
pub fn kernel_main() {
    static USER_PROGRAM: &[u8] = include_bytes!(concat!(
        "../../target/aarch64-unknown-yggdrasil/",
        env!("PROFILE"),
        "/test_program"
    ));

    let devfs_root = devfs::root();
    let tty_node = devfs_root.lookup("ttyS0").unwrap();

    let ioctx = IoContext::new(devfs_root.clone());

    // Spawn a test user task
    let proc =
        proc::exec::create_from_memory(USER_PROGRAM, &["user-program", "argument 1", "argument 2"]);

    match proc {
        Ok(proc) => {
            // Setup I/O for the process
            // let mut io = proc.io.lock();
            // io.set_file(RawFd::STDOUT, todo!()).unwrap();
            {
                let mut io = proc.io.lock();
                io.set_ioctx(ioctx);
                let stdout = tty_node.open(OpenFlags::new().write()).unwrap();
                let stderr = stdout.clone();

                io.set_file(RawFd::STDOUT, stdout).unwrap();
                io.set_file(RawFd::STDERR, stderr).unwrap();
            }

            proc.enqueue_somewhere();
        }
        Err(err) => {
            warnln!("Failed to create user process: {:?}", err);
        }
    };

    Process::current().exit(0);
}
