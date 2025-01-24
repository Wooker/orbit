#![no_std]
#![no_main]
#![feature(const_trait_impl)]
#![allow(unused)]
#![allow(non_camel_case_types)]

pub mod interface;

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
mod riscv32;
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub use crate::riscv32::*;

#[cfg(all(target_arch = "arm", target_os = "none"))]
pub mod cortex_m;
#[cfg(all(target_arch = "arm", target_os = "none"))]
pub use crate::cortex_m as arch;

// #[const_trait]
// pub trait ArchCore {
//     fn new() -> Self;
//     fn init();
//     fn set_freq();
// }
