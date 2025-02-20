#![no_std]
#![no_main]
#![deny(unsafe_code)]

pub mod application;
pub mod service;

use orbit_common::feature_mod;
use orbit_kernel::kernel::Kernel;

extern "Rust" {
    static mut KERNEL: Kernel<4>;
}

feature_mod!("ch592", pub, blinky);
feature_mod!("ch592", pub, uart);

feature_mod!("ch32v208wbu6", pub, blinky_v208);

feature_mod!("ch32v003", pub, blinky_v003);
