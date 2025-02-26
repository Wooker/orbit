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
#[link_section = ".blinky.bss"]
pub static mut BLINKY: Blinky = Blinky {};

#[used]
#[link_section = ".blinky.bss"]
pub static mut STACK: [usize; 1024] = [0; 1024];

pub struct Blinky;
impl Blinky {
    #[inline(never)]
    #[link_section = ".blinky.text"]
    pub fn init(&self) {}
}
impl Application for Blinky {
    #[link_section = ".blinky.text"]
    fn main(&mut self) -> () {
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

    #[inline(never)]
    #[link_section = ".uart.text"]
    unsafe fn stack_top() -> usize {
        STACK.last().unwrap_unchecked() as *const usize as usize + 0x4
    }
}
