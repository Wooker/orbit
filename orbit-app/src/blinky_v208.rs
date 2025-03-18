#![allow(static_mut_refs)]
#![allow(unsafe_code)]
use core::arch::{asm, naked_asm};

use crate::{application::Application, KERNEL};
use orbit_kernel::{
    application::Context,
    arch::interface::timer::Timer,
    chip::pac::GPIOB,
    claim::{Claim, Claimed},
};

#[used]
#[no_mangle]
#[link_section = ".blinky.bss"]
pub static mut BLINKY: Blinky = Blinky::new();

#[used]
#[link_section = ".blinky.bss"]
pub static mut STACK: [usize; 64] = [0; 64];

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
    #[inline(never)]
    #[link_section = ".blinky.text"]
    pub fn init(&mut self) {
        self.context = Context::new();
        self.context.sp = unsafe { STACK.last().unwrap_unchecked() as *const usize as usize + 0x4 };
    }
}
impl Application for Blinky {
    #[link_section = ".blinky.text"]
    fn main(&mut self) -> () {
        let mut gpiob: Claimed<GPIOB> = unsafe { KERNEL.claim().unwrap_unchecked() };
        gpiob.modify(|p| p.cfghr.write(|w| unsafe { w.bits(0b0101) }));

        gpiob.modify(|p| {
            p.bshr.write(|w| unsafe { w.bits(1 << 24) });
            unsafe { KERNEL.core.timer.delay(1000000) };
        });
        gpiob.modify(|p| {
            p.bshr.write(|w| unsafe { w.bits(1 << 8) });
            // unsafe { KERNEL.core.timer.delay(1000000) };
        });
    }

    #[inline(never)]
    #[link_section = ".blinky.text"]
    fn context(&self) -> Context {
        self.context
    }
}
