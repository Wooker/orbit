#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::application::Application;
use orbit_kernel::{arch::entry, kernel::Kernel, kernel::Rate};

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
}

#[entry]
unsafe fn main() -> ! {
    KERNEL.initialize()
}
