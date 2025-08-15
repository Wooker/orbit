use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::{
    arch::delay,
    chip::pac::{GPIOA, SPI1},
};

#[allow(unused)]
use orbit_libos::{
    font::{self, Letter},
    orbit::bitmap,
    spi::{Config, Spi as LibSpi},
};

use crate::app_stack;

static DC_PIN: u8 = 1;
static BUSY_PIN: u32 = 6;

#[allow(unused)]
const HELLO: [Letter; 6] = [font::p, font::r, font::i, font::v, font::e, font::t];

app_stack!(64, "spi");

#[orbit_app(GPIOA, SPI1)]
pub struct Spi {}

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[orbit_impl]
impl Spi {
    #[app_init("spi")]
    pub fn init(&mut self) {
        // MOSI Push-pull alternate output
        // SCK Push-pull alternate output
        self.gpioa.modify(|p| {
            p.cfglr().modify(|r, w| unsafe {
                w.bits(
                    r.bits()
                    | 0b0011 << 0 // PA0 Push-pull output
                    | 0b0011 << 4  // PA1 Push-pull output
                    | 0b1011 << 16 // PA4 NSS Alternate push-pull output
                    | 0b1011 << 20 // PA5 SCK Alternate push-pull output
                    | 0b1000 << 24 // PA6 Pull-up pull-down input
                    | 0b1011 << 28, // PA7 MOSI Alternate push-pull output
                )
            });
            // p.outdr().write(|w| unsafe { w.bits(1 << 4) });
            p.bshr().write(|w| unsafe { w.bits(1 << 0) });
        });
    }

    #[app_interrupt("spi")]
    pub fn interrupt(&mut self) {}

    #[app_main("spi")]
    pub fn main(&mut self) -> Output {
        let mut bus = LibSpi::new(&mut self.spi1, Config::default());
        self.gpioa.modify(|p| {
            p.bshr().write(|w| unsafe { w.bits(1 << 16) });
            delay(1000);
            p.bshr().write(|w| unsafe { w.bits(1 << 0) });
        });

        // while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        Self::cmd(&mut bus, &mut self.gpioa, 0x12);

        Self::cmd(&mut bus, &mut self.gpioa, 0x01);
        Self::data(&mut bus, &mut self.gpioa, 0xc7);
        Self::data(&mut bus, &mut self.gpioa, 0x00);
        Self::data(&mut bus, &mut self.gpioa, 0x00);

        Self::cmd(&mut bus, &mut self.gpioa, 0x11);
        Self::data(&mut bus, &mut self.gpioa, 0b011);

        // For sentence
        // let sentence = HELLO;
        // let cursor_x = 0;
        // let cursor_y = 0;
        // let width = 8 * sentence.len() as u8;
        // let height = 8;

        // For orbit bitmap
        let cursor_x = 0;
        let cursor_y = 0;
        let width = 200;
        let height = 200;

        // RAM X address start end
        Self::cmd(&mut bus, &mut self.gpioa, 0x44);
        Self::data(&mut bus, &mut self.gpioa, cursor_x >> 3);
        Self::data(
            &mut bus,
            &mut self.gpioa,
            (cursor_x >> 3) + (width >> 3) - 1,
        );

        // RAM Y address start end
        Self::cmd(&mut bus, &mut self.gpioa, 0x45);
        Self::data(&mut bus, &mut self.gpioa, cursor_y);
        Self::data(&mut bus, &mut self.gpioa, 0x00);
        Self::data(&mut bus, &mut self.gpioa, cursor_y + (height));
        Self::data(&mut bus, &mut self.gpioa, 0x00);

        Self::cmd(&mut bus, &mut self.gpioa, 0x22);
        Self::data(&mut bus, &mut self.gpioa, 0xb1);

        Self::cmd(&mut bus, &mut self.gpioa, 0x20);

        delay(10000);
        while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        // RAM X address counter
        Self::cmd(&mut bus, &mut self.gpioa, 0x4e);
        Self::data(&mut bus, &mut self.gpioa, cursor_x >> 3);

        // RAM Y address counter
        Self::cmd(&mut bus, &mut self.gpioa, 0x4f);
        Self::data(&mut bus, &mut self.gpioa, cursor_y);
        Self::data(&mut bus, &mut self.gpioa, 0x00);

        Self::cmd(&mut bus, &mut self.gpioa, 0x24);

        // For SENTENCE
        // for slice in 0..8 {
        //     for letter in sentence {
        //         Self::data(&mut bus, &mut self.gpioa, 0xff - letter[slice]);
        //     }
        // }

        // For orbit bitmap
        for p in 0..(((width as usize) >> 3) * (height as usize)) {
            // Self::data(&mut bus, &mut self.gpioa, 0xff);
            // Self::data(&mut bus, &mut self.gpioa, 0x00);
            // Self::data(&mut bus, &mut self.gpioa, 0b10101010);
            // Self::data(&mut bus, &mut self.gpioa, 0b01010101);
            Self::data(&mut bus, &mut self.gpioa, 0xff - bitmap[p]);
        }

        Self::cmd(&mut bus, &mut self.gpioa, 0x22);
        Self::data(&mut bus, &mut self.gpioa, 0xc7);

        Self::cmd(&mut bus, &mut self.gpioa, 0x20);

        delay(10000);
        while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        Self::cmd(&mut bus, &mut self.gpioa, 0x10);
        Self::data(&mut bus, &mut self.gpioa, 0x01);

        Output([0])
    }

    #[link_section = concat!(".", "spi", ".text")]
    pub fn cmd(bus: &mut LibSpi, gpioa: &mut Claimed<GPIOA>, value: u8) {
        Self::set_dc(gpioa, DC_PIN, false);
        bus.write_8(value);
        bus.reset();
    }

    #[link_section = concat!(".", "spi", ".text")]
    pub fn data(bus: &mut LibSpi, gpioa: &mut Claimed<GPIOA>, value: u8) {
        Self::set_dc(gpioa, DC_PIN, true);
        bus.write_8(value);
        bus.reset();
    }

    #[link_section = concat!(".", "spi", ".text")]
    pub fn set_dc(gpio: &mut Claimed<GPIOA>, num: u8, state: bool) {
        let bit = if state { num } else { num + 16 };
        gpio.modify(|p| {
            p.bshr().write(|w| unsafe { w.bits(1 << bit) });
        });
    }
}
