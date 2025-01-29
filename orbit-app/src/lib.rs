#![no_std]
#![no_main]
#![deny(unsafe_code)]

pub mod application;

pub mod blinky;
// pub mod hal_test;

use orbit_kernel::kernel::Kernel;

extern "Rust" {
    static mut KERNEL: Kernel;
}
