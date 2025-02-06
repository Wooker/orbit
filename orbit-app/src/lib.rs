#![no_std]
#![no_main]
#![deny(unsafe_code)]

pub mod application;

use orbit_common::feature_mod;
use orbit_kernel::kernel::Kernel;

extern "Rust" {
    static mut KERNEL: Kernel;
}

feature_mod!("ch592", pub, blinky);
feature_mod!("ch592", pub, uart);

feature_mod!("ch32v208wbu6", pub, blinky_v208);
