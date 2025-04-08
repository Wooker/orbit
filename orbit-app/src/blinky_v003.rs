#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{app_stack, application::Application, KERNEL};
use core::arch::{asm, naked_asm};
use core::sync::atomic::compiler_fence;
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};

use orbit_kernel::{arch::interface::timer::Timer, chip::pac::GPIOA};

app_stack!(32, "blinky");

#[orbit_app(GPIOA)]
pub struct Blinky {}
impl Blinky {
    #[app_init("blinky")]
    pub fn init(&mut self) {
        let gpioa = unsafe { self.gpioa.assume_init_mut() };

        let offset = 1;

        gpioa.modify(|p| {
            p.cfglr
                .write(|w| unsafe { w.bits(0b0011 << (offset << 2)) });
            p.bshr.write(|w| unsafe { w.bits(1 << (offset + 16)) });
        });
    }

    #[app_interrupt("blinky")]
    pub fn interrupt(&mut self) {}
}

#[app_main("blinky", Blinky)]
fn main(&mut self) {
    let gpioa = unsafe { self.gpioa.assume_init_mut() };

    // PA1 to push-pull output
    let offset = 1;

    gpioa.modify(|p| {
        p.cfglr
            .write(|w| unsafe { w.bits(0b0011 << (offset << 2)) })
    });

    unsafe { KERNEL.core.timer.delay(200000) };
    gpioa.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << offset) });
    });
    unsafe { KERNEL.core.timer.delay(200000) };
    gpioa.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << (offset + 16)) });
    });
}
