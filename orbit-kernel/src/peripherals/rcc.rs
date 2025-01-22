pub struct RCC<'a> {
    peripheral: &'a chip::RCC,
}

impl<'a> RCC<'a> {
    pub fn new(peripheral: &'a chip::RCC) -> Self {
        Self { peripheral }
    }
    pub fn reset(&self, bits: u32) {
        self.peripheral.apb2prstr.write(|w| unsafe { w.bits(bits) });
        self.peripheral
            .apb2prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(bits)) });
    }
    pub fn enable_clock(&self, bits: u32) {
        self.peripheral.apb2pcenr.write(|w| unsafe { w.bits(bits) });
    }
}
