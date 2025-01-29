#![no_std]
#![no_main]

pub mod blinky;

use orbit_kernel::kernel::Kernel;

extern "Rust" {
    static mut KERNEL: Kernel;
}
