//! UART: Uni
#![allow(unused)]

// Default UART is UART4()
use crate::chip::PortPeripheral;
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
    pub fn blocking_write_byte(&mut self, b: u8) {}

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn read_byte(&mut self) -> u8 {
        0
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn status(&mut self) -> u32 {
        0
    }

    #[rustc_align(4)]
    #[inline(never)]
    #[unsafe(link_section = ".kernel.text")]
    pub fn clear_int(&mut self, bit: u8) {}
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
