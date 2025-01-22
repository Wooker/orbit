use chip::gpioa::RegisterBlock;

pub struct GPIOA<'a> {
    peripheral: &'a chip::GPIOA,
}
impl<'a> GPIOA<'a> {
    pub fn new(port: &'a chip::GPIOA) -> Self {
        Self { peripheral: port }
    }
    pub fn enable(&self) {
        self.peripheral
            .cfghr
            .write(|w| unsafe { w.bits(0b0010 << 4) });
    }
}

pub struct GPIOB<'a> {
    peripheral: &'a chip::GPIOB,
}
impl<'a> GPIOB<'a> {
    pub fn new(port: &'a chip::GPIOB) -> Self {
        Self { peripheral: port }
    }
    pub fn enable(&self) {
        self.peripheral.cfghr.write(|w| unsafe { w.bits(0b0010) });
    }
}

pub struct GPIO<P, N> {
    port: P,
    pin: N,
}
