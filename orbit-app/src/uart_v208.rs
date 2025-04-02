#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
    sync::atomic::compiler_fence,
};

use orbit_kernel::{
    application::Context,
    arch::interface::timer::Timer,
    claim::{Claim, Claimed},
};
use orbit_libos::uart_v208::{Config, Uart};
use serde::Deserialize;
use serde::Serialize;

#[cfg(feature = "ch32v208wbu6")]
use orbit_kernel::chip::pac::{GPIOC, UART4};

#[cfg(feature = "ch32v003")]
use orbit_kernel::chip::pac::{GPIOD, USART1};

use crate::{app_stack, app_struct, application::Application, KERNEL};

#[allow(unused)]
#[cfg(feature = "ch32v003")]
type UartInstance = USART1;
#[cfg(feature = "ch32v003")]
type GPIOInstance = GPIOD;

#[allow(unused)]
#[cfg(feature = "ch32v208wbu6")]
type UartInstance = UART4;
#[cfg(feature = "ch32v208wbu6")]
type GPIOInstance = GPIOC;

app_struct!(UART_APP: UartApp = UartApp::new(), "uart");
app_stack!(64, "uart");

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
enum Message {
    Hello,
    Bye,
    Unknown,
}
impl From<u8> for Message {
    fn from(value: u8) -> Self {
        match value {
            0 => Message::Hello,
            1 => Message::Bye,
            _ => Message::Unknown,
        }
    }
}
impl Into<u8> for Message {
    fn into(self) -> u8 {
        match self {
            Message::Hello => 0,
            Message::Bye => 1,
            Message::Unknown => u8::MAX,
        }
    }
}

#[repr(C, align(4))]
pub struct UartApp<'u> {
    context: Context,
    buf: [u8; 32],
    read: Message,
    count: usize,
    write: Message,
    uart: MaybeUninit<Uart<'u>>,
    gpio: MaybeUninit<Claimed<'u, GPIOInstance>>,
}

impl<'u> UartApp<'u> {
    #[inline(never)]
    #[link_section = ".uart.text"]
    const fn new() -> Self {
        let context = Context::new();
        Self {
            context,
            buf: [0; 32],
            write: Message::Hello,
            read: Message::Unknown,
            count: 0,
            uart: MaybeUninit::uninit(),
            gpio: MaybeUninit::uninit(),
        }
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    pub fn init(&mut self) {
        extern "C" {
            static _app_uart_text_start: usize;
            static _app_uart_text_end: usize;
            static _app_uart_bss_start: usize;
            static _app_uart_bss_end: usize;
            static _app_uart_text_main: usize;
            static _app_uart_bss_struct: usize;
        }
        let provides = unsafe {
            &_app_uart_text_end as *const usize as usize
                | &_app_uart_text_start as *const usize as usize
                | &_app_uart_text_end as *const usize as usize
                | &_app_uart_bss_start as *const usize as usize
                | &_app_uart_bss_end as *const usize as usize
                | &_app_uart_text_main as *const usize as usize
                | &_app_uart_bss_struct as *const usize as usize
        };
        self.context = Context::new();
        self.context.t0 = provides;
        compiler_fence(core::sync::atomic::Ordering::SeqCst);

        self.context.t0 = 0;
        self.context.sp = unsafe { STACK.last().unwrap_unchecked() as *const usize as usize + 0x4 };
        self.context.gp = &self.context as *const Context as usize;
        self.context.ra = Self::ecall as *const fn() as usize;

        for i in 0..32 {
            self.buf[i] = 0;
        }
        self.count = 0;
        self.write = Message::Hello;
        self.read = Message::Unknown;

        self.uart.write(Uart::new(
            unsafe { KERNEL.claim().unwrap_unchecked() },
            Config::default(),
        ));

        self.gpio
            .write(unsafe { KERNEL.claim().unwrap_unchecked() });
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    pub fn interrupt(&mut self) {
        let uart = unsafe { self.uart.assume_init_mut() };

        uart.read(&mut self.buf[0]);
        self.read = Message::from(self.buf[0]);

        if self.read == Message::Hello {
            self.write = Message::Bye;
            uart.blocking_write_char(self.write.into());
        }

        unsafe { asm!("li a0, -1; li a1, 0;") };
    }
}

impl<'u> Application for UartApp<'u> {
    #[link_section = ".uart.text.main"]
    fn main(&mut self) {
        // PC11 RX as Floating input
        // PC10 TX as push-pull alternate output
        #[cfg(feature = "ch32v208wbu6")]
        unsafe {
            self.gpio.assume_init_mut().modify(|p| {
                p.cfghr.write(|w| w.bits(0b1011 << 8 | 0b1000 << 12));
                p.outdr.write(|w| w.bits(1 << 11));
            })
        };

        // PD6 RX as pull-up input
        // PD5 TX as push-pull multiplexed output
        #[cfg(feature = "ch32v003")]
        unsafe {
            self.gpio.assume_init_mut().modify(|p| {
                p.cfglr
                    .write(|w| unsafe { w.bits(0b1000 << 24 | 0b1011 << 20) });
                p.outdr.write(|w| unsafe { w.bits(1 << 6) });
            })
        };

        let uart = unsafe { self.uart.assume_init_mut() };

        uart.blocking_write_char(self.write.into());
        if self.write == Message::Bye {
            self.write = Message::Hello;
        }
        unsafe { KERNEL.core.timer.delay(1000000) };
        unsafe { asm!("li a0, 0;li a1, 0;") };
    }

    #[inline(never)]
    #[link_section = ".uart.text"]
    fn context(&self) -> Context {
        self.context
    }
    #[naked]
    #[link_section = ".uart.text"]
    extern "C" fn ecall() {
        unsafe { naked_asm!("ecall") };
    }
}
