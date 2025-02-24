use fugit::HertzU32 as Hertz;

// No HSI
const HSE_FREQUENCY: Hertz = Hertz::from_raw(32_000_000);
const PLL_FREQUENCY: Hertz = Hertz::from_raw(480_000_000);

#[no_mangle]
static mut CLOCK: Clocks = Clocks {
    // Power on default
    hclk: Hertz::from_raw(0),
};

/// 32K clock source
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub enum Clock32KSrc {
    #[default]
    LSI,
    LSE,
}

#[derive(Clone, Copy, Debug, PartialEq)]
#[repr(u8)]
pub enum ClockSrc {
    // CK32K
    Clock32K,
    // CK32M from HSE, then div, 2 <= div <= 32
    HSE(u8),
    // CK32M from PLL, then div, 2 <= div <= 32
    PLL(u8),
}

impl Default for ClockSrc {
    fn default() -> Self {
        Self::PLL(8)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct ClockConfig {
    pub clock32ksrc: Clock32KSrc,
    pub mux: ClockSrc,
}

impl ClockConfig {
    pub fn clock_source_lsi() -> Self {
        Self {
            mux: ClockSrc::Clock32K,
            clock32ksrc: Clock32KSrc::LSI,
        }
    }

    pub fn pll_60mhz() -> Self {
        Self {
            mux: ClockSrc::PLL(8),
            ..Default::default()
        }
    }

    pub fn pll_80mhz() -> Self {
        Self {
            mux: ClockSrc::PLL(6),
            ..Default::default()
        }
    }

    pub fn use_lse(mut self) -> Self {
        self.clock32ksrc = Clock32KSrc::LSE;
        self
    }

    pub fn freeze(self) {
        let rcc = unsafe { &*chip::pac::RCC::PTR };
        let extend = unsafe { &*chip::pac::EXTEND::PTR };

        // Reset GPIOB GPIOC
        let gpios = 1 << 3 | 1 << 4;
        rcc.apb2prstr.write(|w| unsafe { w.bits(gpios) });
        rcc.apb2prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(gpios)) });

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

        unsafe {
            CLOCK = Clocks {
                hclk: Hertz::from_raw(8_000_000),
            };
        }
    }
}

#[no_mangle]
pub fn PLLRDY() {}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Clocks {
    pub hclk: Hertz,
}

pub fn clocks() -> &'static Clocks {
    unsafe { &CLOCK }
}
