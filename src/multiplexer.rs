use esp_hal::{
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{Peripherals, AES, DMA, SYSTEM},
};

pub struct Multiplexer {
    aes: PeripheralRef<'static, AES>,
    dma: PeripheralRef<'static, DMA>,
    // peripherals: Peripherals,
}

impl Multiplexer {
    pub fn new(peripherals: &mut Option<Peripherals>) -> Self {
        let peripherals = Option::take(peripherals).unwrap();
        Multiplexer {
            // peripherals: Option::take(peripherals).unwrap(),
            dma: peripherals.DMA.into_ref(),
            aes: peripherals.AES.into_ref(),
        }
    }
}

impl Multiplexer {
    // pub fn system<T: Peripheral<P = SYSTEM>>(self) -> PeripheralRef<'static, SYSTEM> {
    //     self.peripherals.SYSTEM.into_ref()
    // }

    pub fn aes(&self) -> &PeripheralRef<'static, AES> {
        &self.aes
    }

    pub fn dma(&self) -> &PeripheralRef<'static, DMA> {
        &self.dma
    }
}
