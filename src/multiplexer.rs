use esp_hal::{
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{Peripherals, AES, SYSTEM},
};

pub struct Multiplexer {
    peripherals: Peripherals,
}

impl Multiplexer {
    pub fn new(peripherals: &mut Option<Peripherals>) -> Self {
        Multiplexer {
            peripherals: Option::take(peripherals).unwrap(),
        }
    }
}

impl Multiplexer {
    pub fn system<T: Peripheral<P = SYSTEM>>(self) -> PeripheralRef<'static, SYSTEM> {
        self.peripherals.SYSTEM.into_ref()
    }

    pub fn aes<T: Peripheral<P = AES>>(self) -> PeripheralRef<'static, AES> {
        self.peripherals.AES.into_ref()
    }
}
