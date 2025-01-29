#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use orbit_app::blinky::blinky_main;
use orbit_arch::entry;
use orbit_kernel::kernel::Kernel;

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
}

#[entry]
fn main() -> ! {
    unsafe { KERNEL.initialize() };
    blinky_main()
}
