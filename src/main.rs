//! osdev-x kernel crate
#![feature(
    naked_functions,
    asm_const,
    panic_info_message,
    optimize_attribute,
    const_trait_impl,
    maybe_uninit_slice
)]
#![allow(clippy::new_without_default)]
#![warn(missing_docs)]
#![no_std]
#![no_main]

extern crate alloc;

#[macro_use]
pub mod debug;
#[macro_use]
pub mod arch;

pub mod device;
pub mod mem;
pub mod panic;
pub mod sync;
pub mod task;
pub mod util;
