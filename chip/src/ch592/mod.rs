pub use crate::gpio::{OrbitGPIO, OrbitGPIOBuilder};
pub use ch59x::ch59x::Peripherals;

use core::{marker::PhantomData, ptr::null_mut};

pub struct GPIO<const Port: char, const Num: u8> {}
impl<const Port: char, const N: u8> GPIO<Port, N> {
    pub fn new() -> Self {
        Self {}
    }
}
impl<const Port: char, const N: u8> OrbitGPIO<Port, N> for GPIO<Port, N> {
    fn enable(self) -> impl OrbitGPIOBuilder<Port, N> {
        let gpio = unsafe { &*(ch59x::ch59x::GPIO::ptr()) };

        let offset = match N {
            0..=15 => N,
            _ => todo!(),
        };
        match Port {
            'A' => gpio.pa_dir.write(|w| unsafe { w.bits(1 << offset) }),
            'B' => gpio.pb_dir.write(|w| unsafe { w.bits(1 << offset) }),
            _ => {}
        };
        gpio.pa_out.write(|w| unsafe { w.bits(1 << offset) });

        self
    }
    fn disable(&self) {}
}

impl<const Port: char, const N: u8> OrbitGPIOBuilder<Port, N> for GPIO<Port, N> {
    fn configure(self) -> impl OrbitGPIO<Port, N> {
        self
    }
}

pub type PA8 = GPIO<'A', 8>;
pub type PB23 = GPIO<'B', 23>;
pub type PBAD = GPIO<'D', 23>;
