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

use crate::{application::Application, KERNEL};

#[used]
#[no_mangle]
pub static mut UART_APP: UartApp<'static> = UartApp::new();

pub struct UartApp<'a> {
    buf: MaybeUninit<&'a [u8]>,
}
impl<'a> UartApp<'a> {
    const fn new() -> Self {
        Self {
            buf: MaybeUninit::uninit(),
        }
    }
    #[inline(never)]
    pub fn set_buf(&mut self, buf: &'a [u8]) {
        self.buf.write(buf);
    }
}
impl<'a> Application for UartApp<'a> {
    fn main(&self) {
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
            uart.blocking_write("Hello world".as_bytes());
            // uart.write(unsafe { self.buf.assume_init() });
            unsafe { KERNEL.core.timer.delay(1000000) };
        }
    }
}
