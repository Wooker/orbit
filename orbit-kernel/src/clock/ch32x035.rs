use fugit::HertzU32 as Hertz;

#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Clocks {
    pub hclk: Hertz,
}

impl Clocks {
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn freeze(&mut self) {
        let rcc = unsafe { &*chip::pac::RCC::PTR };

        // PA, PB
        let bits = 1 << 3 | 1 << 2;
        unsafe {
            rcc.apb2prstr().write(|w| w.bits(bits));
            rcc.apb2prstr().modify(|r, w| w.bits(r.bits() & !(bits)));
            rcc.apb2pcenr().write(|w| w.bits(bits));
        }

        // USART2
        let bits = 1 << 17;
        unsafe {
            rcc.apb1prstr().write(|w| w.bits(bits));
            rcc.apb1prstr().modify(|r, w| w.bits(r.bits() & !(bits)));
            rcc.apb1pcenr().write(|w| w.bits(bits));
        }

        self.hclk = Hertz::from_raw(48_000_000);
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub const fn default() -> Self {
        Self {
            hclk: Hertz::from_raw(0),
        }
    }
}
