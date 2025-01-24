pub use crate::gpio::OrbitGPIO;
pub use ch59x::{Reg, RegisterSpec, ch59x as pac};

pub struct GPIO<const PORT: char, const NUM: u8> {}
impl<const PORT: char, const NUM: u8> GPIO<PORT, NUM> {
    pub fn new() -> Self {
        let gpio = unsafe { &*(ch59x::ch59x::GPIO::ptr()) };
        gpio.pa_clr.write(|w| unsafe { w.bits(0xffff) });
        gpio.pb_clr.write(|w| unsafe { w.bits(0xffff) });

        gpio.pa_clr.write(|w| unsafe { w.bits(0) });
        gpio.pb_clr.write(|w| unsafe { w.bits(0) });

        Self {}
    }
    pub fn set_high(&mut self) {
        let gpio = unsafe { &*(ch59x::ch59x::GPIO::ptr()) };
        let offset = match NUM {
            0..=15 => NUM,
            _ => todo!(),
        };
        match PORT {
            'A' => gpio.pa_out.write(|w| unsafe { w.bits(1 << offset) }),
            'B' => gpio.pb_out.write(|w| unsafe { w.bits(1 << offset) }),
            _ => {}
        }
    }
    pub fn set_low(&mut self) {
        let gpio = unsafe { &*(ch59x::ch59x::GPIO::ptr()) };
        let offset = match NUM {
            0..=15 => NUM,
            _ => todo!(),
        };
        match PORT {
            'A' => gpio
                .pa_out
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << offset)) }),
            'B' => gpio
                .pb_out
                .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << offset)) }),
            _ => {}
        }
    }
}
impl<const PORT: char, const NUM: u8> OrbitGPIO<PORT, NUM> for GPIO<PORT, NUM> {
    fn enable(&mut self) {
        let gpio = unsafe { &*(ch59x::ch59x::GPIO::ptr()) };

        let offset = match NUM {
            0..=15 => NUM,
            _ => todo!(),
        };
        match PORT {
            'A' => gpio.pa_dir.write(|w| unsafe { w.bits(1 << offset) }),
            'B' => gpio.pb_dir.write(|w| unsafe { w.bits(1 << offset) }),
            _ => {}
        };
    }
    fn disable(&mut self) {}
    fn configure(&mut self) {}
}

pub type PA8 = GPIO<'A', 8>;
pub type PB23 = GPIO<'B', 23>;
pub type PBAD = GPIO<'D', 23>;
