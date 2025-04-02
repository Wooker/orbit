#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{application::Application, KERNEL};
use core::arch::{asm, naked_asm};
use core::sync::atomic::compiler_fence;

use orbit_kernel::{
    application::Context,
    arch::interface::timer::Timer,
    chip::pac::GPIOD,
    claim::{Claim, Claimed},
};

#[used]
#[no_mangle]
#[link_section = ".blinky.bss"]
pub static BLINKY: Blinky = Blinky::new();

#[used]
#[link_section = ".blinky.bss"]
pub static mut STACK: [usize; 32] = [0; 32];

pub struct Blinky {
    context: Context,
}
impl Blinky {
    pub const fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }
    pub fn init(&mut self) {
        extern "C" {
            static _app_blinky_text_start: usize;
            static _app_blinky_text_end: usize;
            static _app_blinky_bss_start: usize;
            static _app_blinky_bss_end: usize;
            static _app_blinky_text_main: usize;
            static _app_blinky_bss_struct: usize;
        }
        let provides = unsafe {
            &_app_blinky_text_end as *const usize as usize
                | &_app_blinky_text_start as *const usize as usize
                | &_app_blinky_text_end as *const usize as usize
                | &_app_blinky_bss_start as *const usize as usize
                | &_app_blinky_bss_end as *const usize as usize
                | &_app_blinky_text_main as *const usize as usize
                | &_app_blinky_bss_struct as *const usize as usize
        };
        self.context = Context::new();
        self.context.t0 = provides;
        compiler_fence(core::sync::atomic::Ordering::SeqCst);

        self.context.t0 = 0;
        self.context.sp = unsafe { STACK.last().unwrap_unchecked() as *const usize as usize + 0x4 };
        self.context.gp = &self.context as *const Context as usize;
        self.context.ra = Self::ecall as *const fn() as usize;
        self.context.a0 = self as *const Blinky as usize;
    }
    #[naked]
    #[link_section = ".uart.text"]
    unsafe extern "C" fn ecall() {
        naked_asm!("ecall");
    }
}
impl Application for Blinky {
    #[inline(never)]
    #[link_section = ".blinky.text"]
    fn main(&mut self) {
        let mut gpio: Claimed<GPIOD> = unsafe { KERNEL.claim().unwrap_unchecked() };

        // PD7 to push-pull output
        let offset = 7;

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

    #[inline(never)]
    #[link_section = ".blinky.text"]
    fn context(&self) -> Context {
        self.context
    }
}
