#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use core::fmt::Write;
use core::mem::MaybeUninit;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;

use orbit_kernel::{
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

pub struct UartApp {
    buf: [u8; 32],
}

impl<'a> UartApp {
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
    fn main(&mut self) {
        let mut gpiod: Claimed<GPIOD> = unsafe { KERNEL.claim().unwrap_unchecked() };

        // PD6 RX as floating input
        // PD5 TX as push-pull multiplexed output
        gpiod.modify(|p| {
            p.cfglr
                .write(|w| unsafe { w.bits(0b0100 << 24 | 0b1011 << 20) })
        });

        let mut uart1: Claimed<USART1> = unsafe { KERNEL.claim().unwrap_unchecked() };
        let mut uart = Uart::new(uart1, Config::default());
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
