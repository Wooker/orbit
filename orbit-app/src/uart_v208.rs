#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use core::arch::asm;
use core::fmt::Write;
use core::mem::MaybeUninit;
use core::ptr::null;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

use orbit_kernel::{
    arch::interface::timer::Timer,
    chip::pac::{GPIOB, GPIOC, UART4},
    claim::{Claim, Claimed},
};
use orbit_libos::uart_v208::{Config, Uart};
use postcard::to_slice;
use serde::Deserialize;
use serde::Serialize;

use crate::{application::Application, KERNEL};

#[used]
#[no_mangle]
#[link_section = ".uart.bss"]
pub static mut UART_APP: UartApp = UartApp::new();

#[used]
#[link_section = ".uart.bss"]
pub static mut STACK: [usize; 1024] = [0; 1024];

#[derive(Serialize, Deserialize)]
enum Message<'m> {
    Hello,
    Str(&'m str),
}

pub struct UartApp {
    buf: [u8; 32],
}

impl UartApp {
    #[inline(never)]
    #[link_section = ".uart.text"]
    const fn new() -> Self {
        Self { buf: [0; 32] }
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    pub fn init(&mut self) {
        for b in self.buf.iter_mut() {
            *b = 0;
        }
    }
}

impl Application for UartApp {
    #[link_section = ".uart.text"]
    fn main(&mut self) {
        let mut gpioc: Claimed<GPIOC> = unsafe { KERNEL.claim().unwrap_unchecked() };

        // PC10 TX as push-pull alternate output
        // PC 11 RX as Floating input
        gpioc.modify(|p| {
            p.cfghr
                .write(|w| unsafe { w.bits(0b1011 << 8 | 0b0100 << 12) })
        });

        let mut uart4: Claimed<UART4> = unsafe { KERNEL.claim().unwrap_unchecked() };
        let mut uart = Uart::new(uart4, Config::default());
        let str = "Hello";

        for (i, &b) in "Hello".as_bytes().iter().enumerate() {
            self.buf[i] = b;
        }
        loop {
            uart.blocking_write(unsafe {
                to_slice(&Message::Str("Hello"), &mut self.buf).unwrap_unchecked()
            });
            unsafe { KERNEL.core.timer.delay(1000000) };
        }
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    unsafe fn stack_top() -> usize {
        STACK.last().unwrap_unchecked() as *const usize as usize + 0x4
    }
}
