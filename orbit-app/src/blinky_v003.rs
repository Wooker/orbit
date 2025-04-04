#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{app_stack, app_struct, application::Application, KERNEL};
use core::arch::{asm, naked_asm};
use core::sync::atomic::compiler_fence;
use orbit_common_proc_macro::{app_init, app_interrupt, app_main};

use orbit_kernel::{
    application::Context,
    arch::interface::timer::Timer,
    chip::pac::GPIOA,
    claim::{Claim, Claimed},
};

app_struct!(BLINKY: Blinky = Blinky::new(), "blinky");
app_stack!(32, "blinky");

pub struct Blinky {
    context: Context,
}
impl Blinky {
    pub const fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }
    #[app_init("blinky")]
    pub fn init(&mut self) {}

    #[app_interrupt("blinky")]
    pub fn interrupt(&mut self) {}
}

#[app_main("blinky", Blinky)]
fn main(&mut self) {
    let mut gpio: Claimed<GPIOA> = unsafe { KERNEL.claim().unwrap_unchecked() };

    // PD7 to push-pull output
    let offset = 1;

    gpio.modify(|p| {
        p.cfglr
            .write(|w| unsafe { w.bits(0b0011 << (offset << 2)) })
    });

    unsafe { KERNEL.core.timer.delay(200000) };
    gpio.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << offset) });
    });
    unsafe { KERNEL.core.timer.delay(200000) };
    gpio.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << (offset + 16)) });
    });
}
