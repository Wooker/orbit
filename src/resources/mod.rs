use esp_hal::peripherals::{AES, DMA, SYSTEM};

pub mod resource;

pub struct Resources {
    pub AES: AES,
    pub DMA: DMA,
}
