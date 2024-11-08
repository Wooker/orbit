#![no_std]
#![no_main]
#![feature(const_trait_impl)]

#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub mod riscv32;
#[cfg(all(target_arch = "riscv32", target_os = "none"))]
pub use crate::riscv32 as arch;

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
