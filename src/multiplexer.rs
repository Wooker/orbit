use esp_hal::{
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{Peripherals, AES, SYSTEM},
};

pub struct Resource<'res, T: Peripheral<P = T>> {
    inner: PeripheralRef<'res, T>,
}
impl<'res, T: Peripheral<P = T>> Resource<'res, T> {
    pub fn inner(self) -> PeripheralRef<'res, T> {
        self.inner
    }
}

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

pub trait Claim {
    type Item;

    fn claim(self) -> PeripheralRef<'static, Self::Item>
    where
        <Self as Claim>::Item: Peripheral<P = Self::Item>;
}

impl Claim for Resource<'static, AES> {
    type Item = AES;

    fn claim(self) -> PeripheralRef<'static, AES> {
        self.inner
    }
}

impl Claim for Resource<'static, SYSTEM> {
    type Item = SYSTEM;

    fn claim(self) -> PeripheralRef<'static, SYSTEM> {
        self.inner
    }
}
