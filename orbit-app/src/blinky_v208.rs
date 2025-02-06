#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{application::Application, KERNEL};
use orbit_kernel::{
    arch::interface::timer::Timer,
    chip::pac::GPIOB,
    claim::{Claim, Claimed},
};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static BLINKY: Blinky = Blinky {};

pub struct Blinky;
impl Application<1> for Blinky {
    fn main(&self) {
        let mut gpiob: Claimed<GPIOB> = unsafe { KERNEL.claim().unwrap_unchecked() };
        gpiob.modify(|p| p.cfghr.write(|w| unsafe { w.bits(0b0101) }));

        loop {
            gpiob.modify(|p| {
                p.bshr.write(|w| unsafe { w.bits(1 << 24) });
                unsafe { KERNEL.core.timer.delay(1000000) };
            });
            gpiob.modify(|p| {
                p.bshr.write(|w| unsafe { w.bits(1 << 8) });
                unsafe { KERNEL.core.timer.delay(1000000) };
            });
        }
    }
}
