#[cfg(feature = "qingke_v4")]
pub mod qingke;
#[cfg(feature = "qingke_v4")]
pub use qingke_rt::entry;

#[cfg(feature = "esp_riscv")]
pub use esp_riscv_rt::entry;

#[cfg(feature = "riscv")]
pub use riscv;
#[cfg(feature = "riscv")]
pub use riscv_rt::entry;
