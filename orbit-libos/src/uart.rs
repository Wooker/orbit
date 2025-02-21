#![allow(static_mut_refs)]
//! UART: Uni

// Default UART is UART1(PA8/PA9)
use orbit_kernel::{
    chip::pac::UART1,
    claim::{Claim, Claimed},
    clock::ch592::clocks,
    kernel::Kernel,
};

unsafe extern "Rust" {
    static mut KERNEL: Kernel<4>;
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Parity {
    ParityNone = 0xff,
    ParityEven = 0b01,
    ParityOdd = 0b00,
    ParityMark = 0b10,  // 1
    ParitySpace = 0b11, // 0
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
    DataBits5 = 0b00,
    DataBits6 = 0b01,
    DataBits7 = 0b10,
    DataBits8 = 0b11,
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
    uart: Claimed<'a, UART1>,
}

impl<'a> Uart<'a> {
    pub fn new(mut uart: Claimed<'a, UART1>, config: Config) -> Self {
        uart.modify(|p| {
            p.uart1_fcr.write(|w| unsafe {
                w.fcr_rx_fifo_clr()
                    .set_bit()
                    .fcr_rx_fifo_clr()
                    .set_bit()
                    .fcr_fifo_en() // enable FIFO
                    .set_bit()
                    .fcr_fifo_trig()
                    .bits(2) // FIFO trigger on 4 bytes
            })
        });
        uart.modify(|p| {
            p.uart1_lcr
                .write(|w| unsafe { w.lcr_word_sz().bits(config.data_bits as u8) })
        }); // word size set to 8 bits

        uart.modify(|p| {
            p.uart1_lcr.modify(|_, w| match config.stop_bits {
                StopBits::STOP1 => w.lcr_stop_bit().clear_bit(),
                StopBits::STOP2 => w.lcr_stop_bit().set_bit(),
            })
        }); // 1 or 2 stop bits

        match config.parity {
            Parity::ParityNone => {
                uart.modify(|p| p.uart1_lcr.modify(|_, w| w.lcr_par_en().clear_bit()))
            }
            _ => uart.modify(|p| {
                p.uart1_lcr.modify(|_, w| unsafe {
                    w.lcr_par_en()
                        .set_bit()
                        .lcr_par_mod()
                        .bits(config.parity as u8)
                })
            }),
        }

        // baudrate = Fsys * 2 / R8_UARTx_DIV / 16 / R16_UARTx_DL
        let x = 10 * clocks().hclk.to_Hz() / 8 / config.baudrate;
        let x = ((x + 5) / 10) & 0xffff;

        uart.modify(|p| p.uart1_div.write(|w| unsafe { w.bits(1) }));
        uart.modify(|p| p.uart1_dl.write(|w| unsafe { w.bits(x as u16) }));

        // enable TX
        uart.modify(|p| p.uart1_ier.write(|w| w.ier_txd_en().set_bit()));

        Self { uart }
    }

    pub fn blocking_write(&mut self, buf: &[u8]) {
        let uart1 = &mut self.uart;

        const UART_FIFO_SIZE: u8 = 8;

        for &c in buf {
            while uart1.read(|p| p.uart1_tfc.read().uart1_tfc().bits()) >= UART_FIFO_SIZE {
                // wait
            }
            uart1.modify(|p| p.uart1_thr().write(|w| unsafe { w.uart1_rbr().bits(c) }));
        }
    }
}

impl<'a> core::fmt::Write for Uart<'a> {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self.blocking_write(s.as_bytes());
        Ok(())
    }
}
