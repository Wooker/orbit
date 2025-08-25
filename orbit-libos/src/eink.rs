use crate::spi::Spi;
use orbit_kernel::arch::delay;
use orbit_kernel::chip::pac::GPIOA;
use orbit_kernel::claim::Claimed;

pub struct Eink<'a, 'b, const DC_PIN: u8, const BUSY_PIN: u8> {
    bus: Spi<'a, 'b>,
    gpioa: &'a mut Claimed<'b, GPIOA>,
}

impl<'a, 'b, const DC_PIN: u8, const BUSY_PIN: u8> Eink<'a, 'b, DC_PIN, BUSY_PIN> {
    pub fn new(bus: Spi<'a, 'b>, gpioa: &'a mut Claimed<'b, GPIOA>) -> Self {
        gpioa.modify(|p| {
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
            p.bshr().write(|w| unsafe { w.bits(1 << 0) });
        });

        Self { bus, gpioa }
    }

    pub fn display(&mut self, frame: &[u8; 5000], partial: bool) {
        self.gpioa.modify(|p| {
            p.bshr().write(|w| unsafe { w.bits(1 << 16) });
            delay(10000);
            p.bshr().write(|w| unsafe { w.bits(1 << 0) });
        });

        delay(100);
        while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        self.cmd(0x12);

        delay(100);
        while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        self.cmd(0x01);
        self.data(0xc7);
        self.data(0x00);
        self.data(0x00);

        self.cmd(0x11);
        self.data(0b011);

        // For orbit bitmap
        let cursor_x = 0;
        let cursor_y = 0;
        let width = 200;
        let height = 200;

        // RAM X address start end
        self.cmd(0x44);
        self.data(cursor_x >> 3);
        self.data((cursor_x >> 3) + (width >> 3) - 1);

        // RAM Y address start end
        self.cmd(0x45);
        self.data(cursor_y);
        self.data(0x00);
        self.data(cursor_y + (height));
        self.data(0x00);

        self.cmd(0x22);
        self.data(if partial { 0xb9 } else { 0xb1 });

        self.cmd(0x20);

        delay(100);
        while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        // RAM X address counter
        self.cmd(0x4e);
        self.data(cursor_x >> 3);

        // RAM Y address counter
        self.cmd(0x4f);
        self.data(cursor_y);
        self.data(0x00);

        self.cmd(0x24);

        // For orbit bitmap
        for p in 0..(((width as usize) >> 3) * (height as usize)) {
            self.data(0xff - frame[p]);
        }

        self.cmd(0x22);
        self.data(if partial { 0xcf } else { 0xc7 });

        self.cmd(0x20);

        delay(100);
        while self.gpioa.read(|p| p.indr().read().bits()) & (1 << BUSY_PIN) != 0 {}

        self.cmd(0x10);
        self.data(0x01);

        self.bus.reset();
    }

    pub fn cmd(&mut self, value: u8) {
        while self.bus.busy() {}

        self.set_dc(DC_PIN, false);
        self.bus.write_8(value);
    }

    pub fn data(&mut self, value: u8) {
        while self.bus.busy() {}

        self.set_dc(DC_PIN, true);
        self.bus.write_8(value);
    }

    pub fn set_dc(&mut self, num: u8, state: bool) {
        let bit = if state { num } else { num + 16 };
        self.gpioa.modify(|p| {
            p.bshr().write(|w| unsafe { w.bits(1 << bit) });
        });
    }
}
