static mut IRQ_STA: usize = 0;

use core::ptr;

use orbit_arch::register::gintenr;

pub fn with_safe_access<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    const REG_SAFE_ACCESS_SIG: *mut u8 = 0x40001040 as *mut u8;
    const SAFE_ACCESS_SIG1: u8 = 0x57;
    const SAFE_ACCESS_SIG2: u8 = 0xA8;

    unsafe {
        if gintenr::read() & 0x08 != 0 {
            IRQ_STA = gintenr::read();
            gintenr::write(IRQ_STA & (!0x08));
        }
        orbit_arch::riscv::asm::nop();
        orbit_arch::riscv::asm::nop();

        ptr::write_volatile(REG_SAFE_ACCESS_SIG, SAFE_ACCESS_SIG1);
        ptr::write_volatile(REG_SAFE_ACCESS_SIG, SAFE_ACCESS_SIG2);

        orbit_arch::riscv::asm::nop();
        orbit_arch::riscv::asm::nop();
    }
    let ret = f();
    unsafe {
        ptr::write_volatile(REG_SAFE_ACCESS_SIG, 0);
        gintenr::write(gintenr::read() | (IRQ_STA & 0x08));
        IRQ_STA = 0;
        orbit_arch::riscv::asm::nop();
        orbit_arch::riscv::asm::nop();
    }
    ret
}

use fugit::HertzU32 as Hertz;

// No HSI
const HSE_FREQUENCY: Hertz = Hertz::from_raw(32_000_000);
const PLL_FREQUENCY: Hertz = Hertz::from_raw(480_000_000);

static mut CLOCK: Clocks = Clocks {
    // Power on default
    hclk: Hertz::from_raw(6_400_000),
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
        let sysctl = unsafe { &*chip::pac::SYSTEM_CONTROL::PTR };
        let sys = unsafe { &*chip::pac::SYS::PTR };

        match self.clock32ksrc {
            Clock32KSrc::LSE => {
                with_safe_access(|| {
                    sys.ck32k_config.modify(|_, w| w.clk_xt32k_pon().set_bit());
                });
                orbit_arch::riscv::asm::delay(clocks().hclk.to_Hz() / 10 / 4);
                with_safe_access(|| unsafe {
                    sys.xt32k_tune.modify(|_, w| w.xt32k_i_tune().bits(0b01));
                    sys.ck32k_config.modify(|_, w| w.clk_osc32k_xt().set_bit());
                });
                orbit_arch::riscv::asm::delay(clocks().hclk.to_Hz() / 1000);
            }
            _ => (),
        }

        with_safe_access(|| unsafe {
            sysctl
                .pll_config
                .modify(|r, w| w.pll_cfg_dat().bits(r.pll_cfg_dat().bits() & !(1 << 5)));
        });
        let hclk = match self.mux {
            ClockSrc::HSE(div) => {
                assert!(div != 1, "1 means close HCLK");
                with_safe_access(|| unsafe {
                    sys.clk_sys_cfg.write(|w| {
                        w.pll_pwr_en()
                            .set_bit()
                            .xt_32m_pwr_en()
                            .set_bit()
                            .clk_sys_mod()
                            .bits(0b10)
                            .clk_pll_div()
                            .bits(div & 0x1f)
                    });
                    orbit_arch::riscv::asm::nop();
                    orbit_arch::riscv::asm::nop();
                    orbit_arch::riscv::asm::nop();
                    orbit_arch::riscv::asm::nop();
                });
                // with_safe_access(|| unsafe {
                //     sysctl.flash_cfg.modify(|_, w| w.bits(0x51));
                // });
                Hertz::from_raw(HSE_FREQUENCY.to_Hz() / (div as u32))
            }
            ClockSrc::PLL(div) => {
                assert!(div != 1, "1 means close HCLK");
                with_safe_access(|| unsafe {
                    sys.clk_sys_cfg.write(|w| {
                        w.pll_pwr_en()
                            .set_bit()
                            .xt_32m_pwr_en()
                            .set_bit()
                            .clk_sys_mod()
                            .bits(0b01)
                            .clk_pll_div()
                            .bits(div & 0x1f)
                    });
                    orbit_arch::riscv::asm::nop();
                    orbit_arch::riscv::asm::nop();
                    orbit_arch::riscv::asm::nop();
                    orbit_arch::riscv::asm::nop();
                });
                // with_safe_access(|| unsafe {
                //     sysctl.flash_cfg.modify(|_, w| w.bits(0x52));
                // });
                Hertz::from_raw(PLL_FREQUENCY.to_Hz() / (div as u32))
            }
            _ => unimplemented!("CK32K not implemented"),
        };
        with_safe_access(|| unsafe {
            sysctl
                .pll_config
                .modify(|r, w| w.pll_cfg_dat().bits(r.pll_cfg_dat().bits() | (1 << 7)));
        });

        unsafe {
            CLOCK = Clocks { hclk };
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Clocks {
    pub hclk: Hertz,
}

pub fn clocks() -> &'static Clocks {
    unsafe { &CLOCK }
}
