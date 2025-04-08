#![allow(static_mut_refs)]
#![allow(unsafe_code)]
use core::arch::{asm, naked_asm};

use crate::{app_stack, application::Application, KERNEL};
use core::sync::atomic::compiler_fence;
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};
use orbit_kernel::{arch::interface::timer::Timer, chip::pac::GPIOB};

app_stack!(64, "blinky");

#[orbit_app(GPIOB)]
pub struct Blinky {}

impl Blinky {
    #[app_init("blinky")]
    pub fn init(&mut self) {
        let gpiob = unsafe { self.gpiob.assume_init_mut() };
        gpiob.modify(|p| {
            p.cfghr.write(|w| unsafe { w.bits(0b0101) });
            p.bshr.write(|w| unsafe { w.bits(1 << 8) });
        });
    }

    #[app_interrupt("blinky")]
    pub fn interrupt(&mut self) {}
}

#[app_main("blinky", Blinky)]
fn main(&mut self) -> () {
    let gpiob = unsafe { self.gpiob.assume_init_mut() };

    gpiob.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << 24) });
        unsafe { KERNEL.core.timer.delay(100000) };
        p.bshr.write(|w| unsafe { w.bits(1 << 8) });
    });
}
