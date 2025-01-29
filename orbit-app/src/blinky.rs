#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use orbit_kernel::{
    chip::pac::GPIO,
    kernel::claim::{Claim, Claimed},
    orbit_arch::interface::timer::Timer,
};

use crate::{application::Application, KERNEL};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static BLINKY: Blinky = Blinky {};

extern "C" {
    static _sapps: usize;
}

pub struct Blinky;
impl Application<1> for Blinky {
    fn main(&self) {
        let mut gpio: Claimed<GPIO> = unsafe { KERNEL.claim().unwrap_unchecked() };
        loop {
            unsafe { KERNEL.core.timer.delay(200000) };
            gpio.modify(|p| {
                p.pa_dir
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 8)) });
            });
            unsafe { KERNEL.core.timer.delay(200000) };
            gpio.modify(|p| {
                p.pa_dir.write(|w| unsafe { w.bits(1 << 8) });
            });
        }
    }
}
