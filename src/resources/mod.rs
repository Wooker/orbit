use esp_hal::{
    clock::{ClockControl, Clocks},
    peripherals::{AES, DMA, SYSTEM},
    system::SystemClockControl,
};

pub mod resource;

pub struct Resources {
    pub AES: AES,
    pub DMA: DMA,
    // pub clocks: Clocks<'res>,
}
