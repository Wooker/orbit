#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use core::arch::{asm, naked_asm};
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

use orbit_kernel::{
    application::Context,
    arch::interface::timer::Timer,
    chip::pac::{GPIOD, USART1},
    claim::{Claim, Claimed},
};
use orbit_libos::uart_v208::{Config, Uart};
use postcard::to_slice;
use serde::Deserialize;
use serde::Serialize;

use crate::{application::Application, KERNEL};

#[derive(Serialize, Deserialize)]
enum Message<'m> {
    Hello,
    Str(&'m str),
}

#[used]
#[no_mangle]
#[link_section = ".uart.bss"]
pub static mut UART_APP: UartApp = UartApp::new();

#[used]
#[link_section = ".uart.bss"]
pub static mut STACK: [usize; 32] = [0; 32];

#[repr(C, align(4))]
pub struct UartApp {
    context: Context,
    buf: [u8; 32],
}

impl<'a> UartApp {
    #[inline(never)]
    #[link_section = ".uart.text"]
    const fn new() -> Self {
        Self {
            context: Context::new(),
            buf: [0; 32],
        }
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    pub fn init(&mut self) {
        for b in self.buf.iter_mut() {
            *b = 0;
        }
        self.context = Context::new();
        self.context.sp = unsafe { STACK.last().unwrap_unchecked() as *const usize as usize + 0x4 };
        self.context.gp = &self.context as *const Context as usize;
        self.context.ra = Self::ecall as *const fn() as usize;
        self.context.a0 = self as *const UartApp as usize;
    }

    #[naked]
    #[link_section = ".uart.text"]
    unsafe extern "C" fn ecall() {
        naked_asm!("ecall");
    }
}

impl Application for UartApp {
    #[inline(never)]
    #[link_section = ".uart.text"]
    fn main(&mut self) {
        let mut gpiod: Claimed<GPIOD> = unsafe { KERNEL.claim().unwrap_unchecked() };

        // PD6 RX as pull-up input
        // PD5 TX as push-pull multiplexed output
        gpiod.modify(|p| {
            p.cfglr
                .write(|w| unsafe { w.bits(0b1000 << 24 | 0b1011 << 20) });
            p.outdr.write(|w| unsafe { w.bits(1 << 6) });
        });

        let mut uart1: Claimed<USART1> = unsafe { KERNEL.claim().unwrap_unchecked() };
        let mut uart = Uart::new(uart1, Config::default());

        uart.blocking_write(unsafe { to_slice(&Message::Hello, &mut self.buf).unwrap_unchecked() });
        unsafe { KERNEL.core.timer.delay(1000000) };
        unsafe { asm!("li a0, 0;li a1, 0;") };
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    fn context(&self) -> Context {
        self.context
    }
}
