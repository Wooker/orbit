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
#[link_section = ".blinky.bss"]
pub static BLINKY: Blinky = Blinky {};

#[used]
#[link_section = ".blinky.bss"]
pub static mut STACK: [usize; 32] = [0; 32];

pub struct Blinky;
impl Application for Blinky {
    fn main(&mut self) {
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

    #[inline(never)]
    #[link_section = ".uart.text"]
    unsafe fn stack_top() -> usize {
        STACK.last().unwrap_unchecked() as *const usize as usize + 0x4
    }
}
