#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{application::Application, KERNEL};
use orbit_kernel::{
    arch::interface::timer::Timer,
    chip::pac::GPIOA,
    claim::{Claim, Claimed},
};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static BLINKY: Blinky = Blinky {};

extern "C" {
    static _sapps: usize;
}

pub struct Blinky;
impl Application for Blinky {
    fn main(&self) {
        let mut gpio: Claimed<GPIOA> = unsafe { KERNEL.claim().unwrap_unchecked() };
        loop {
            unsafe { KERNEL.core.timer.delay(200000) };
            gpio.modify(|p| {
                p.outdr
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 8)) });
            });
            unsafe { KERNEL.core.timer.delay(200000) };
            gpio.modify(|p| {
                p.outdr.write(|w| unsafe { w.bits(1 << 8) });
            });
        }
    }
}
