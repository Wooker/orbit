#[cfg(feature = "qingke_v4")]
pub mod qingke;
use core::arch::global_asm;

#[cfg(feature = "qingke_v4")]
pub use qingke_rt::entry;

#[cfg(feature = "esp_riscv")]
pub use esp_riscv_rt::entry;

#[cfg(feature = "riscv")]
pub use riscv;
// #[cfg(feature = "riscv")]
// pub use riscv_rt::entry;

#[cfg(feature = "riscv")]
global_asm!(
    "
.global _start;

_start:
   csrwi mie, 0; 
   j kernel_main;
"
);
