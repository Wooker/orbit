use esp_hal::{
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{AES, SYSTEM},
};

use crate::resources::resource::Resource;

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
