//! UART: Uni
#![allow(unused)]

// Default UART is UART4()
use chip::PortPeripheral;
use core::sync::atomic::{Ordering, compiler_fence};
use spaceport::{constants::EOF, transport::Transport};

use super::{ConfigureGPIO, PortKinds};

#[derive(Clone, Copy)]
pub enum Parity {
    ParityNone = 0x00,
    ParityEven = 0b10,
    ParityOdd = 0b11,
}

#[derive(Clone, Copy)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1 = 0b00,
    #[doc = "2 stop bits"]
    STOP2 = 0b10,
}

#[derive(Clone, Copy)]
pub enum DataBits {
    DataBits8 = 0b0,
    DataBits9 = 0b1,
}

#[derive(Clone, Copy)]
pub struct Config {
    pub baudrate: u32,
    pub data_bits: DataBits,
    pub stop_bits: StopBits,
    pub parity: Parity,
}
impl Default for Config {
    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

#[derive(Clone, Copy)]
pub struct Uart<'a> {
    uart: &'a PortPeripheral,
    count: usize,
}

impl<'a> Uart<'a> {
    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn new(uart: &'a PortPeripheral, kind: impl ConfigureGPIO, config: Config) -> Self {
        // Configure GPIO for UART alt-function
        kind.configure();

        // Configure UART peripheral
        uart.ctlr1().write(|w| unsafe {
            // Data bits and parity configuratoin
            let mut ctlr1 = 0_u32;
            ctlr1 |= (config.data_bits as u32) << 12;
            ctlr1 |= (config.parity as u32) << 9;
            ctlr1 |= 1 << 5; // rx interrupt
            ctlr1 |= 1 << 3;
            ctlr1 |= 1 << 2;
            ctlr1 |= 1 << 13;
            w.bits(ctlr1)
        });
        compiler_fence(Ordering::SeqCst);
        uart.ctlr2().write(|w| unsafe {
            // Data bits and parity configuratoin
            let mut ctlr2 = 0_u32;
            ctlr2 |= (config.stop_bits as u32) << 12;
            w.bits(ctlr2)
        });
        compiler_fence(Ordering::SeqCst);
        uart.ctlr3().write(|w| unsafe {
            // Data bits and parity configuratoin
            let mut ctlr3 = 0_u32;
            ctlr3 |= 1;
            w.bits(ctlr3)
        });

        // To calculate div, use the following
        // let clock = unsafe { KERNEL.clock() };
        // let div_m = 25 * clock / (4 * config.baudrate);
        // let mut tmpreg = (div_m / 100) << 4;
        // let div_f = div_m - 100 * (tmpreg >> 4);
        // tmpreg |= ((div_f * 16 + 50) / 100) & 0x0F;

        // With the default clock frequency of 8MHz the
        // value of uart_div is 69
        uart.brr().write(|w| unsafe { w.bits(69) });

        Self { uart, count: 0 }
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn blocking_write(&mut self, buf: &[u8]) {
        for c in buf {
            self.blocking_write_byte(*c);
        }
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn blocking_write_byte(&mut self, b: u8) {
        self.uart.datar().write(|w| unsafe { w.bits(b as u32) });
        while (self.uart.statr().read().bits() & (1 << 6)) == 0 {} // wait tx complete
        self.uart
            .statr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !(0b11111 << 5)) });
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn read_byte(&mut self) -> u8 {
        let val = self.uart.datar().read().dr().bits() as u8;
        for i in 5..=9 {
            self.clear_int(i);
        }
        val
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn status(&mut self) -> u32 {
        self.uart.statr().read().bits()
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn clear_int(&mut self, bit: u8) {
        self.uart
            .statr()
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << bit)) });
    }
}

impl<'a> Transport for Uart<'a> {
    type Error = super::UartError;

    #[inline(never)]
    fn write(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        self.blocking_write(data);
        Ok(())
    }

    #[inline(never)]
    fn read(&mut self, buf: &mut [u8]) -> Result<usize, Self::Error> {
        if buf.len() < self.count + 1 {
            Err(Self::Error::BufTooSmall)
        } else {
            let b = self.read_byte();
            buf[self.count] = b;
            if b != EOF {
                self.count += 1;
                Ok(0)
            } else {
                let count = self.count + 1;
                self.count = 0;
                Ok(count)
            }
        }
    }
}
