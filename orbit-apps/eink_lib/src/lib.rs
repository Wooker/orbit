#![no_std]

use ch32x035_spi_driver::Spi;
use orbit_kernel::arch::delay;
use orbit_kernel::chip::pac::GPIOA;

pub enum Direction {
    XuYdXd = 0b000,
    XuYdXi = 0b001,
    XuYiXd = 0b010,
    XuYiXi = 0b011,
    YuYdXd = 0b100,
    YuYdXi = 0b101,
    YuYiXd = 0b110,
    YuYiXi = 0b111,
}

pub struct Position {
    pub x: usize,
    pub y: usize,
}

pub struct Size {
    pub width: usize,
    pub height: usize,
}

pub struct Config {
    pub pos: Position,
    pub dir: Direction,
    pub size: Size,
}

pub struct Eink<'app, const DC_PIN: u8, const BUSY_PIN: u8> {
    bus: Spi<'app>,
    gpioa: &'app mut GPIOA,
}

impl<'app, const DC_PIN: u8, const BUSY_PIN: u8> Eink<'app, DC_PIN, BUSY_PIN> {
    #[inline(never)]
    pub fn new(bus: Spi<'app>, gpioa: &'app mut GPIOA) -> Self {
        gpioa.cfglr().modify(|r, w| unsafe {
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
        gpioa.bshr().write(|w| unsafe { w.bits(1 << 0) });

        Self { bus, gpioa }
    }

    #[inline(never)]
    pub fn display(&mut self, config: Config, frame: &[u8; 5000], partial: bool) {
        self.gpioa.bshr().write(|w| unsafe { w.bits(1 << 16) });
        delay(10000);
        self.gpioa.bshr().write(|w| unsafe { w.bits(1 << 0) });

        delay(100);
        while self.gpioa.indr().read().bits() & (1 << BUSY_PIN) != 0 {}

        self.cmd(0x12);

        delay(100);
        while self.gpioa.indr().read().bits() & (1 << BUSY_PIN) != 0 {}

        self.cmd(0x01);
        self.data(0xc7);
        self.data(0x00);
        self.data(0x00);

        self.cmd(0x11);
        self.data(config.dir as u8);

        // RAM X address start end
        self.cmd(0x44);
        self.data((config.pos.x >> 3) as u8);
        self.data(((config.pos.x >> 3) + (config.size.width >> 3) - 1) as u8);

        // RAM Y address start end
        self.cmd(0x45);
        self.data(config.pos.y as u8);
        self.data(0x00);
        self.data((config.pos.y + (config.size.height)) as u8);
        self.data(0x00);

        self.cmd(0x22);
        self.data(if partial { 0xb9 } else { 0xb1 });

        self.cmd(0x20);

        delay(100);
        while self.gpioa.indr().read().bits() & (1 << BUSY_PIN) != 0 {}

        // RAM X address counter
        self.cmd(0x4e);
        self.data((config.pos.x >> 3) as u8);

        // RAM Y address counter
        self.cmd(0x4f);
        self.data(config.pos.y as u8);
        self.data(0x00);

        self.cmd(0x24);

        // For orbit bitmap
        for p in 0..((config.size.width >> 3) * config.size.height) {
            self.data(0xff - frame[p]);
        }

        self.cmd(0x22);
        self.data(if partial { 0xcf } else { 0xc7 });

        self.cmd(0x20);

        delay(100);
        while self.gpioa.indr().read().bits() & (1 << BUSY_PIN) != 0 {}

        self.cmd(0x10);
        self.data(0x01);

        self.bus.reset();
    }

    #[inline(never)]
    pub fn cmd(&mut self, value: u8) {
        while self.bus.busy() {}

        self.set_dc(DC_PIN, false);
        self.bus.write_8(value);
    }

    #[inline(never)]
    pub fn data(&mut self, value: u8) {
        while self.bus.busy() {}

        self.set_dc(DC_PIN, true);
        self.bus.write_8(value);
    }

    #[inline(never)]
    pub fn set_dc(&mut self, num: u8, state: bool) {
        let bit = if state { num } else { num + 16 };
        self.gpioa.bshr().write(|w| unsafe { w.bits(1 << bit) });
    }
}
