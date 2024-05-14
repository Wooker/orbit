use esp_hal::{
    clock::ClockControl,
    peripherals::{AES, DMA, SYSTEM},
};

pub mod resource;

pub struct Resources<'d> {
    pub AES: AES,
    pub DMA: DMA,
    pub CLOCK_CONTROL: ClockControl<'d>,
    // pub SYSTEM: SYSTEM,
}
