#![allow(static_mut_refs)]
//! UART: Uni

// Default UART is UART4()
use crate::KERNEL;
use core::sync::atomic::compiler_fence;
use core::sync::atomic::Ordering;
use orbit_kernel::{
    claim::{Claim, Claimable, Claimed},
    kernel::Kernel,
};

#[cfg(feature = "ch32v003")]
use orbit_kernel::chip::pac::USART1;

#[cfg(feature = "ch32v208wbu6")]
use orbit_kernel::chip::pac::UART4;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone = 0x00,
    ParityEven = 0b10,
    ParityOdd = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1 = 0b00,
    #[doc = "2 stop bits"]
    STOP2 = 0b10,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DataBits {
    DataBits8 = 0b0,
    DataBits9 = 0b1,
}

pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}
impl Default for Config {
    #[inline(never)]
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

#[cfg(feature = "ch32v003")]
type Instance = USART1;

#[cfg(feature = "ch32v208wbu6")]
type Instance = UART4;

pub struct Uart<'a> {
    uart: Claimed<'a, Instance>,
}

impl<'a> Uart<'a> {
    #[inline(never)]
    pub fn new(mut uart: Claimed<'a, Instance>, config: Config) -> Self {
        uart.modify(|p| {
            p.ctlr1.write(|w| unsafe {
                // Data bits and parity configuratoin
                let mut ctlr1 = 0_u32;
                ctlr1 |= (config.data_bits as u32) << 12;
                ctlr1 |= (config.parity as u32) << 9;
                ctlr1 |= 0b11111 << 4; // interrupts
                ctlr1 |= 1 << 3;
                ctlr1 |= 1 << 2;
                ctlr1 |= 1 << 13;
                w.bits(ctlr1)
            });
        });
        compiler_fence(Ordering::SeqCst);
        uart.modify(|p| {
            p.ctlr2.write(|w| unsafe {
                // Data bits and parity configuratoin
                let mut ctlr2 = 0_u32;
                ctlr2 |= (config.stop_bits as u32) << 12;
                w.bits(ctlr2)
            });
        });

        // let clock = unsafe { KERNEL.clock() };
        // let div_m = 25 * clock / (4 * config.baudrate);
        // let mut tmpreg = (div_m / 100) << 4;
        // let div_f = div_m - 100 * (tmpreg >> 4);
        // tmpreg |= ((div_f * 16 + 50) / 100) & 0x0F;

        // With the default clock frequency of 8MHz the
        // value of uart_div is 69
        uart.modify(|p| p.brr.write(|w| unsafe { w.bits(69) }));

        Self { uart }
    }

    #[inline(never)]
    pub fn blocking_write(&mut self, buf: &[u8]) {
        for c in buf {
            // Read TC
            while self.uart.read(|p| p.statr.read().bits() & (1 << 6)) == 0 {} // wait tx complete
            self.uart
                .modify(|p| p.datar.write(|w| unsafe { w.bits(*c as u32) }));
        }

        self.uart.modify(|p| {
            p.statr
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 6)) })
        });
    }
}
