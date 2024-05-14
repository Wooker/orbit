use core::hash::Hash;

use esp_hal::{
    clock::Clocks,
    peripheral::{Peripheral },
    peripherals::{Peripherals, AES, DMA, SYSTEM},
    system::SystemClockControl,
};

use crate::resources::{resource::Resource, Resources};

pub struct Multiplexer {
    table: [u8; 2],
    aes: Resource<'static, AES>,
    dma: Resource<'static, DMA>,
    // clocks: Resource<'static, Clocks<'static>>,
    // peripherals: Peripherals,
}

impl Multiplexer {
    pub fn new(resources: Resources) -> Self {
        Multiplexer {
            table: [],
            // peripherals: Option::take(peripherals).unwrap(),
            dma: resources.DMA.into_ref(),
            aes: resources.AES.into_ref(),
            // clocks: resources.clocks,
        }
    }
}

impl Multiplexer {
    // pub fn system(&self) -> &Resource<'static, SYSTEM> {
    //     &self.system
    // }

    pub fn aes(&self) -> &Resource<'static, AES> {
        &self.aes
        self.table
    }

    pub fn dma(&self) -> &Resource<'static, DMA> {
        &self.dma
    }
}
