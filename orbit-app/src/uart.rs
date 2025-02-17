#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use core::fmt::Write;
use core::mem::MaybeUninit;

use orbit_kernel::{
    arch::interface::timer::Timer,
    chip::pac::UART1,
    claim::{Claim, Claimed},
};
use orbit_libos::uart::{Config, Uart};

use crate::{application::Application, KERNEL};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static UART_APP: UartApp = UartApp {
    data: MaybeUninit::uninit(),
};

extern "C" {
    static _sapps: usize;
}

pub struct UartApp<'a> {
    data: MaybeUninit<&'a str>,
}
impl<'a> UartApp<'a> {
    fn print(&self, uart: &mut Uart) {
        unsafe { uart.write_str(self.data.assume_init()).unwrap_unchecked() };
    }
    pub fn init(&mut self, data: &'a str) {
        self.data.write(data);
    }
}
impl<'a> Application<1> for UartApp<'a> {
    fn main(&self) {
        let uart1: Claimed<UART1> = unsafe { KERNEL.claim().unwrap_unchecked() };
        let mut uart = Uart::new(uart1, Config::default());
        loop {
            self.print(&mut uart);
            unsafe { KERNEL.core.timer.delay(200000) };
        }
    }
}
