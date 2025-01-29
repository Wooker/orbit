#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky::Blinky};
use orbit_kernel::{arch::entry, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
    static mut BLINKY: Blinky;
}

#[entry]
unsafe fn main() -> ! {
    KERNEL.initialize();
    BLINKY.main();
    // KERNEL.schedule();
    loop {}
}
