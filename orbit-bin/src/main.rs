#![no_std]
#![no_main]
#![allow(static_mut_refs)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky::Blinky};
use orbit_arch::entry;
use orbit_kernel::kernel::Kernel;

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
    static mut BLINKY: Blinky;
}

#[entry]
fn main() -> ! {
    unsafe {
        KERNEL.initialize();
        BLINKY.main();
    }
    loop {}
}
