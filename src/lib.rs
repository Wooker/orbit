#![no_std]
#![no_main]

#[cfg(feature = "esp32-c3")]
pub use esp_hal::efuse;
#[cfg(feature = "esp32-c3")]
use esp_hal::peripherals::Peripherals;

pub mod kernel;
use kernel::Resource;

#[cfg(feature = "esp32-c3")]
impl Resource for Peripherals {}
