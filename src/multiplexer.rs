use esp_hal::{
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{Peripherals, AES, DMA, SYSTEM},
};

use crate::resources::Resources;

pub struct Multiplexer {
    aes: PeripheralRef<'static, AES>,
    dma: PeripheralRef<'static, DMA>,
    // peripherals: Peripherals,
}

impl Multiplexer {
    pub fn new(resources: Resources) -> Self {
        Multiplexer {
            // peripherals: Option::take(peripherals).unwrap(),
            dma: resources.DMA.into_ref(),
            aes: resources.AES.into_ref(),
        }
    }
}

impl Multiplexer {
    // pub fn system(&self) -> &PeripheralRef<'static, SYSTEM> {
    //     &self.system
    // }

    pub fn aes(&self) -> &PeripheralRef<'static, AES> {
        &self.aes
    }

    pub fn dma(&self) -> &PeripheralRef<'static, DMA> {
        &self.dma
    }
}
