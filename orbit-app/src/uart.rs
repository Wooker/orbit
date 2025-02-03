#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use core::fmt::Write;

use orbit_kernel::{
    arch::interface::timer::Timer,
    chip::pac::{GPIO, UART1},
    kernel::claim::{Claim, Claimed},
    kernel::clock::ClockConfig,
};
use orbit_libos::uart::{Config, Uart};

use crate::{application::Application, KERNEL};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static UART_APP: UartApp = UartApp {};

extern "C" {
    static _sapps: usize;
}

pub struct UartApp;
impl Application<1> for UartApp {
    fn main(&self) {
        ClockConfig::pll_60mhz().freeze();
        let mut gpio: Claimed<GPIO> = unsafe { KERNEL.claim().unwrap_unchecked() };
        gpio.modify(|p| {
            p.pa_dir.write(|w| unsafe { w.bits(1 << 9) });
            p.pa_pu.write(|w| unsafe { w.bits(1 << 8) });
        });

        gpio.modify(|p| {
            p.pa_out
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 8)) });
        });

        let uart1: Claimed<UART1> = unsafe { KERNEL.claim().unwrap_unchecked() };
        let mut uart = Uart::new(uart1, Config::default());
        loop {
            gpio.modify(|p| {
                p.pa_out
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 8)) });
            });

            unsafe { uart.write_str("Hello world!\n").unwrap_unchecked() };
            unsafe { KERNEL.core.timer.delay(200000) };

            gpio.modify(|p| {
                p.pa_out
                    .modify(|r, w| unsafe { w.bits(r.bits() & (1 << 8)) });
            });
            unsafe { KERNEL.core.timer.delay(200000) };
        }
    }
}
