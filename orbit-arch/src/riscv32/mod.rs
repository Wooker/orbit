use core::arch::global_asm;

#[cfg(feature = "qingke_v4")]
mod qingke_cpu;
#[cfg(feature = "qingke_v4")]
pub use qingke::*;
#[cfg(feature = "qingke_v4")]
pub use qingke_cpu::Core;
#[cfg(feature = "qingke_v4")]
pub use qingke_rt::entry;

#[cfg(feature = "esp_riscv")]
pub use esp_riscv_rt::*;
#[cfg(feature = "esp_riscv")]
mod esp_riscv;

// #[cfg(feature = "riscv")]
// pub use riscv_rt_macros::entry;

// #[cfg(feature = "riscv")]
// pub use riscv;

// #[cfg(feature = "riscv")]
// pub use riscv_rt::entry;
