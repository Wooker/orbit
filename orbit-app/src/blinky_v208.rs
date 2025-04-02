#![allow(static_mut_refs)]
#![allow(unsafe_code)]
use core::arch::{asm, naked_asm};

use crate::{application::Application, KERNEL};
use core::sync::atomic::compiler_fence;
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
    #[inline(never)]
    #[link_section = ".blinky.text"]
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
    }

    #[inline(never)]
    #[link_section = ".blinky.text"]
    pub fn interrupt(&mut self) {
        unsafe { asm!("li a0, -1; li a1, 0;") };
    }
}
impl Application for Blinky {
    #[link_section = ".blinky.text.main"]
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
        unsafe { asm!("li a0, 0;li a1, 0;") };
    }

    #[inline(never)]
    #[link_section = ".blinky.text"]
    fn context(&self) -> Context {
        self.context
    }
    #[naked]
    #[link_section = ".blinky.text"]
    extern "C" fn ecall() {
        unsafe { naked_asm!("ecall") };
    }
}
