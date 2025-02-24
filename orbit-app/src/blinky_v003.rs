#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{application::Application, KERNEL};
use orbit_kernel::{
    arch::interface::timer::Timer,
    chip::pac::GPIOD,
    claim::{Claim, Claimed},
};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static BLINKY: Blinky = Blinky {};

pub struct Blinky;
impl Application for Blinky {
    fn main(&self) {
        let mut gpio: Claimed<GPIOD> = unsafe { KERNEL.claim().unwrap_unchecked() };

        // PD7 to push-pull output
        let offset = 6;

        gpio.modify(|p| p.cfglr.write(|w| unsafe { w.bits(0b0011 << 24) }));

        loop {
            unsafe { KERNEL.core.timer.delay(200000) };
            gpio.modify(|p| {
                p.bshr.write(|w| unsafe { w.bits(1 << offset) });
            });
            unsafe { KERNEL.core.timer.delay(200000) };
            gpio.modify(|p| {
                p.bshr.write(|w| unsafe { w.bits(1 << offset + 16) });
            });
        }
    }
}
