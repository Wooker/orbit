use core::arch::global_asm;

#[cfg(any(feature = "qingke_v4", feature = "qingke_v2"))]
mod qingke_cpu;
#[cfg(any(feature = "qingke_v4", feature = "qingke_v2"))]
pub use qingke::*;
#[cfg(any(feature = "qingke_v4", feature = "qingke_v2"))]
pub use qingke_cpu::Core;
#[cfg(any(feature = "qingke_v4", feature = "qingke_v2"))]
pub use qingke_rt::entry;

#[cfg(feature = "esp_riscv")]
pub use esp_riscv_rt::*;
#[cfg(feature = "esp_riscv")]
mod esp_riscv;

// #[cfg(feature = "riscv")]
// pub use riscv_rt_macros::entry;

#[cfg(feature = "riscv")]
pub use riscv;

#[cfg(feature = "riscv")]
pub use riscv_rt::entry;

#[cfg(feature = "riscv")]
mod riscv_cpu;

#[cfg(feature = "riscv")]
pub use riscv_cpu::Core;
