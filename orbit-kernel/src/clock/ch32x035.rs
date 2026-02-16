#[derive(Copy, Clone, Eq, PartialEq)]
pub struct Clocks {
    pub hclk: usize,
}

impl Clocks {
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn freeze(&mut self) {
        let rcc = unsafe { &*chip::pac::RCC::PTR };

        // USART2
        let bits = 1 << 17;
        unsafe {
            rcc.apb1prstr().write(|w| w.bits(bits));
            rcc.apb1prstr().modify(|r, w| w.bits(r.bits() & !(bits)));
            rcc.apb1pcenr().write(|w| w.bits(bits));
        }

        // PA, PB, SPI1
        let bits = 1 << 3 | 1 << 2 | 1 << 12;
        unsafe {
            rcc.apb2prstr().write(|w| w.bits(bits));
            rcc.apb2prstr().modify(|r, w| w.bits(r.bits() & !(bits)));
            rcc.apb2pcenr().write(|w| w.bits(bits));
        }

        self.hclk = 48_000_000;
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub const fn default() -> Self {
        Self { hclk: 0 }
    }
}
