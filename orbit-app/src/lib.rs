#![no_std]
#![no_main]
#![deny(unsafe_code)]

pub mod application;

#[cfg(feature = "ch592")]
pub mod blinky;
#[cfg(feature = "ch592")]
pub mod uart;

use orbit_kernel::kernel::Kernel;

extern "Rust" {
    static mut KERNEL: Kernel;
}
