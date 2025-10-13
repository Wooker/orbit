#![no_std]
#![allow(unused)]

use core::{
    mem::MaybeUninit,
    sync::atomic::{Ordering, compiler_fence},
};

use orbit_kernel::{
    arch::{delay, interface::timer::Timer},
    chip::pac::SPI1,
    claim::{Claim, Claimed},
    kernel::Kernel,
};

#[derive(Copy, Clone)]
enum DataSize {
    _16 = 0b1,
    _8 = 0b0,
}

#[derive(Copy, Clone)]
enum FirstBit {
    LSB = 0b1,
    MSB = 0b0,
}

#[derive(Copy, Clone)]
enum BaudRatePre {
    _2 = 0b000,
    _4 = 0b001,
    _8 = 0b010,
    _16 = 0b011,
    _32 = 0b100,
    _64 = 0b101,
    _256 = 0b111,
}

#[derive(Copy, Clone)]
enum Mode {
    Slave = 0b0,
    Master = 0b1,
}

#[derive(Copy, Clone)]
enum Direction {
    OneWireTX,
    OneWireRX,
    TwoWire,
}

pub struct Config {
    dir: Direction,
    mode: Mode,
    data_size: DataSize,
    cpol: bool,
    cpha: bool,
    nss: bool,
    baud: BaudRatePre,
    first: FirstBit,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            dir: Direction::TwoWire,
            mode: Mode::Master,
            data_size: DataSize::_8,
            cpol: false,
            cpha: false,
            nss: true,
            baud: BaudRatePre::_2,
            first: FirstBit::MSB,
        }
    }
}

pub struct Spi<'app, 'claim> {
    spi: &'app mut Claimed<'claim, SPI1>,
    ctlr1: u16,
}

impl<'app, 'claim> Spi<'app, 'claim>
where
    'claim: 'app,
{
    pub fn new(spi: &'app mut Claimed<'claim, SPI1>, config: Config) -> Self {
        let mut bits = 0;
        spi.modify(|p| {
            bits = match config.dir {
                Direction::OneWireTX => 1 << 15 | 1 << 14,
                Direction::OneWireRX => 1 << 15,
                Direction::TwoWire => 0,
            };
            bits = bits
                | (config.data_size as u16) << 11
                | 0b11 << 8  // Software control CE
                // | 0b1 << 9  // Software control CE
                // | 1 << 8
                | (config.first as u16) << 7 | (config.baud as u16) << 3
                | (config.mode as u16) << 2 | (config.cpol as u16) << 1 | config.cpha as u16;
            p.ctlr1().modify(|r, w| unsafe { w.bits(bits) });

            p.ctlr2().modify(|r, w| unsafe { w.bits(1 << 2) });

            // Enable SPI
            p.ctlr1()
                .modify(|r, w| unsafe { w.bits(r.bits() | (1 << 6)) });
        });

        Self { spi, ctlr1: bits }
    }

    pub fn cs_toggle(&mut self) {
        self.spi.modify(|p| {
            p.ctlr1()
                .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 8)) })
        });
    }

    pub fn write_8(&mut self, data: u8) {
        if self.spi.read(|p| (p.statr().read().bits() & 0x1) as u32) == 1 {
            let _ = self.read_8();
        }

        while !self.spi.read(|p| (p.statr().read().txe().bit_is_set())) {
            if self.spi.read(|p| (p.statr().read().modf().bit_is_set())) {
                self.spi.modify(|p| {
                    p.ctlr1().modify(|r, w| unsafe { w.bits(self.ctlr1) });
                });
            }
        }

        self.spi
            .modify(|p| p.datar().write(|w| unsafe { w.bits(data as u16) }));
        // self.spi.read(|p| p.crcr().read().bits().into());
    }
    pub fn read_8(&mut self) -> u8 {
        self.spi.read(|p| p.datar().read().bits() as u32) as u8
    }

    pub fn busy(&self) -> bool {
        self.spi.read(|p| p.statr().read().bsy().bit_is_set())
    }

    pub fn reset(&mut self) {
        delay(1000);
        self.spi.modify(|p| {
            p.ctlr1()
                .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 6)) })
        });
        delay(1000);
        self.spi.modify(|p| {
            p.ctlr1()
                .modify(|r, w| unsafe { w.bits(r.bits() ^ (1 << 6)) })
        });
    }
}
