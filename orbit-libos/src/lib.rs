#![no_std]
#![no_main]

use orbit_common::feature_mod;
use orbit_kernel::kernel::Kernel;

unsafe extern "Rust" {
    pub static mut KERNEL: Kernel<'static>;
}

feature_mod!("ch592", pub, uart);
// feature_mod!("ch32v208wbu6", pub, uart_v208);

// #[cfg(any(feature = "ch32v208wbu6", feature = "ch32v003"))]
// pub mod uart_v208;
