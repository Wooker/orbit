#![no_std]
#![no_main]
#![deny(unsafe_code)]

pub mod application;
pub mod service;

use orbit_common::feature_mod;
use orbit_kernel::kernel::Kernel;

#[allow(unsafe_code)]
unsafe extern "Rust" {
    #[cfg(feature = "ch32v208wbu6")]
    static mut KERNEL: Kernel<4>;

    #[cfg(feature = "ch32v003")]
    static mut KERNEL: Kernel<0>;
}

feature_mod!("ch592", pub, blinky);
feature_mod!("ch592", pub, uart);

feature_mod!("ch32v208wbu6", pub, blinky_v208);
// feature_mod!("ch32v208wbu6", pub, uart_v208);
#[cfg(feature = "ch32v208wbu6")]
pub mod uart_v208;

feature_mod!("ch32v003", pub, blinky_v003);

#[cfg(feature = "ch32v003")]
pub mod uart_v003;
