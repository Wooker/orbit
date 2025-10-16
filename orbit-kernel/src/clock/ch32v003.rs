use fugit::HertzU32 as Hertz;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Clocks {
    pub hclk: Hertz,
}

impl Clocks {
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn freeze(&mut self) {
        let rcc = unsafe { &*chip::pac::RCC::PTR };

        let uart_pd = 1 << 14 | 1 << 5;
        rcc.apb2prstr.write(|w| unsafe { w.bits(uart_pd) });
        rcc.apb2prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(uart_pd)) });

        rcc.apb2pcenr.write(|w| unsafe { w.bits(uart_pd) });

        // HSI is on by default
        self.hclk = Hertz::from_raw(8_000_000);
    }

    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub const fn default() -> Self {
        Self {
            hclk: Hertz::from_raw(0),
        }
    }
}
