use esp_hal::{
    clock::{ClockControl, Clocks},
    peripherals::{AES, DMA, GPIO, IO_MUX, SYSTEM},
    system::SystemClockControl,
    IO,
};

pub mod resource;

pub struct Resources {
    pub AES: AES,
    pub DMA: DMA,
    pub GPIO: GPIO,
    pub IO_MUX: IO_MUX,
    // pub clocks: Clocks<'res>,
}
