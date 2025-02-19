#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use core::arch::asm;

use orbit_app::{application::Application, blinky_v208::Blinky};
use orbit_kernel::{arch::entry, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
    static mut BLINKY: Blinky;
}

#[entry]
fn main() -> ! {
    unsafe { KERNEL.add_application(0, Blinky::main as usize) };
    unsafe { KERNEL.initialize() };
    loop {}
}
