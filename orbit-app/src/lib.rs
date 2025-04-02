#![no_std]
#![no_main]
#![deny(unsafe_code)]
#![feature(naked_functions)]

pub mod application;
pub mod service;

use orbit_common::feature_mod;
use orbit_kernel::kernel::Kernel;

#[allow(unsafe_code)]
unsafe extern "Rust" {
    pub static mut KERNEL: Kernel<'static>;
}

pub mod wfi;

feature_mod!("ch592", pub, blinky);
feature_mod!("ch592", pub, uart);

feature_mod!("ch32v208wbu6", pub, blinky_v208);
// feature_mod!("ch32v208wbu6", pub, uart_v208);

#[cfg(any(feature = "ch32v208wbu6", feature = "ch32v003"))]
pub mod uart_v208;

feature_mod!("ch32v003", pub, blinky_v003);

// #[cfg(feature = "ch32v003")]
// pub mod uart_v003;

/// Macros

#[macro_export]
macro_rules! app_struct {
    ($name:ident: $type:ty = $value:expr, $app_name:expr) => {
        #[used]
        #[no_mangle]
        #[link_section = concat!(".", $app_name, ".bss.struct")]
        pub static mut $name: $type = $value;
    };
}

#[macro_export]
macro_rules! app_stack {
    ($size:expr, $app_name:expr) => {
        #[used]
        #[link_section = concat!(".", $app_name, ".bss")]
        pub static mut STACK: [usize; $size] = [0; $size];
    };
}
