#![allow(static_mut_refs)]
//! UART: Uni

// Default UART is UART4()
use orbit_kernel::{
    chip::pac::UART4,
    claim::{Claim, Claimed},
    clock::ch32v208wbu6::clocks,
    kernel::Kernel,
};

unsafe extern "Rust" {
    static mut KERNEL: Kernel<4>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone = 0x00,
    ParityEven = 0b10,
    ParityOdd = 0b11,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum StopBits {
    #[doc = "1 stop bit"]
    STOP1,
    #[doc = "2 stop bits"]
    STOP2,
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
    fn default() -> Self {
        Self {
            baudrate: 115200,
            data_bits: DataBits::DataBits8,
            stop_bits: StopBits::STOP1,
            parity: Parity::ParityNone,
        }
    }
}

pub struct Uart<'a> {
    uart: Claimed<'a, UART4>,
}

impl<'a> Uart<'a> {
    pub fn new(mut uart: Claimed<'a, UART4>, config: Config) -> Self {
        uart.modify(|p| {
            p.ctlr2.write(|w| unsafe {
                // Data bits and parity configuratoin
                w.bits((config.data_bits as u32) << 12 | (config.parity as u32) << 9)
            });
        });
        uart.modify(|p| {
            p.ctlr1
                // Enable TX and RX
                .modify(|r, w| unsafe { w.bits(r.bits() | 0b11 << 2) });
        });

        Self { uart }
    }

    pub fn blocking_write(&mut self, buf: &[u8]) {
        let uart = &mut self.uart;

        const UART_FIFO_SIZE: u8 = 8;

        for &c in buf {
            // Read RXNE
            while uart.read(|p| p.statr.read().bits() & 0b1 << 5) != 0 {
                // wait
            }
            uart.modify(|p| p.datar.write(|w| unsafe { w.bits(c as u32) }));
        }
    }
}

impl<'a> core::fmt::Write for Uart<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.blocking_write(s.as_bytes());
        Ok(())
    }
}
