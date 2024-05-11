use esp_hal::peripheral::{Peripheral, PeripheralRef};

pub struct Resource<'res, T: Peripheral<P = T>> {
    pub inner: PeripheralRef<'res, T>,
}
impl<'res, T: Peripheral<P = T>> Resource<'res, T> {
    pub fn inner(self) -> PeripheralRef<'res, T> {
        self.inner
    }
}
