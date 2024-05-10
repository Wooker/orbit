pub struct Resource<'res, T: Peripheral<P = T>> {
    inner: PeripheralRef<'res, T>,
}
impl<'res, T: Peripheral<P = T>> Resource<'res, T> {
    pub fn inner(self) -> PeripheralRef<'res, T> {
        self.inner
    }
}
