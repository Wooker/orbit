#![allow(static_mut_refs)]
#![allow(unsafe_code)]
use core::arch::{asm, naked_asm};

use crate::{app_stack, app_struct, application::Application, KERNEL};
use core::sync::atomic::compiler_fence;
use orbit_common_proc_macro::{app_init, app_interrupt, app_main};
use orbit_kernel::{
    application::Context,
    arch::interface::timer::Timer,
    chip::pac::GPIOB,
    claim::{Claim, Claimed},
};
app_struct!(BLINKY: Blinky = Blinky::new(), "blinky");
app_stack!(64, "blinky");

#[repr(C, align(4))]
pub struct Blinky {
    context: Context,
}
impl Blinky {
    #[inline(never)]
    #[link_section = ".blinky.text"]
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
fn main(&mut self) -> () {
    let mut gpiob: Claimed<GPIOB> = unsafe { KERNEL.claim().unwrap_unchecked() };
    gpiob.modify(|p| p.cfghr.write(|w| unsafe { w.bits(0b0101) }));

    gpiob.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << 24) });
        unsafe { KERNEL.core.timer.delay(1000000) };
    });
    gpiob.modify(|p| {
        p.bshr.write(|w| unsafe { w.bits(1 << 8) });
        unsafe { KERNEL.core.timer.delay(1000000) };
    });
}
