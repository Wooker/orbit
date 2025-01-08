#![no_std]
#![no_main]

#[cfg(feature = "esp32c3")]
pub mod esp32c3;

#[cfg(feature = "ch32v208wbu6")]
pub mod ch32v208wbu6;
