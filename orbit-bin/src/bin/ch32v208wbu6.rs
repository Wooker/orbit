#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky_v208::Blinky};
use orbit_kernel::{arch::entry, kernel::Kernel, kernel::Rate};

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
    static mut BLINKY: Blinky;
}

#[entry]
unsafe fn main() -> ! {
    KERNEL.initialize(Rate::<u32, 1, 1>::MHz(60));
    // BLINKY.main();
    // KERNEL.schedule();
    loop {}
}
