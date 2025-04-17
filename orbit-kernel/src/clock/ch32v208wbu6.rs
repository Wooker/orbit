use fugit::HertzU32 as Hertz;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Clocks {
    pub hclk: Hertz,
}

impl Clocks {
    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn freeze(&mut self) {
        let rcc = unsafe { &*chip::pac::RCC::PTR };
        let extend = unsafe { &*chip::pac::EXTEND::PTR };

        // Reset GPIOB GPIOC
        let gpios = 1 << 3 | 1 << 4;
        unsafe {
            rcc.apb2prstr.write(|w| w.bits(gpios));
            rcc.apb2prstr.modify(|r, w| w.bits(r.bits() & !(gpios)));
        }

        // Enable GPIOB GPIOC
        rcc.apb2pcenr.write(|w| unsafe { w.bits(gpios) });

        extend.extend_ctr.write(|w| unsafe { w.bits(1 << 4) }); // set hsipre

        rcc.ctlr.write(|w| unsafe { w.bits(1 << 24) }); // PLLON
        while !rcc.ctlr.read().pllrdy().bit_is_set() {}

        // Reset UART4
        let uart4_rst_bit = 1 << 19;
        rcc.apb1prstr.write(|w| unsafe { w.bits(uart4_rst_bit) });
        rcc.apb1prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(uart4_rst_bit)) });

        // Enable UART4
        rcc.apb1pcenr.write(|w| unsafe { w.bits(uart4_rst_bit) });

        self.hclk = Hertz::from_raw(8_000_000);
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub const fn default() -> Self {
        Self {
            hclk: Hertz::from_raw(0),
        }
    }
}
