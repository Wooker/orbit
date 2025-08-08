use chip::{
    pac::{USART1, USART2, USART3, USART4},
    PortPeripheral,
};
use orbit_common_proc_macro::define_ports;

use crate::port::ConfigureGPIO;

/// USART1 = 32
/// USART2 = 39
/// USART3 = 42
/// USART4 = 43
define_ports!(USART2 = 39);

#[derive(Clone, Copy)]
pub(crate) enum PortKinds {
    USART1,
    USART2,
    USART3,
    USART4,
}

impl ConfigureGPIO for PortKinds {
    fn configure(&self) {
        match self {
            PortKinds::USART1 => {
                // PB11 RX as Floating input
                // PB10 TX as push-pull alternate output
                let gpio = unsafe { &*chip::pac::GPIOB::PTR };
                unsafe {
                    gpio.cfghr().write(|w| w.bits(0b1011 << 8 | 0b1000 << 12));
                    gpio.outdr().write(|w| w.bits(1 << 11));
                };
            }
            PortKinds::USART2 => {
                // PA3 RX as Floating input
                // PA2 TX as push-pull alternate output
                let gpio = unsafe { &*chip::pac::GPIOA::PTR };
                unsafe {
                    gpio.cfglr().write(|w| w.bits(0b1011 << 8 | 0b1000 << 12));
                    gpio.outdr().write(|w| w.bits(1 << 3));
                };
            }
            PortKinds::USART3 => {
                // PB4 RX as Floating input
                // PB3 TX as push-pull alternate output
                let gpio = unsafe { &*chip::pac::GPIOB::PTR };
                unsafe {
                    gpio.cfglr().write(|w| w.bits(0b1011 << 12 | 0b1000 << 16));
                    gpio.outdr().write(|w| w.bits(1 << 4));
                };
            }
            PortKinds::USART4 => {
                // PB1 RX as Floating input
                // PB0 TX as push-pull alternate output
                let gpio = unsafe { &*chip::pac::GPIOB::PTR };
                unsafe {
                    gpio.cfglr().write(|w| w.bits(0b1011 | 0b1000 << 4));
                    gpio.outdr().write(|w| w.bits(1 << 1));
                };
            }
        }
    }
}
