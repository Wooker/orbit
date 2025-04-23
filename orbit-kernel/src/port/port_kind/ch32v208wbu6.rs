use chip::{
    pac::{UART4, USART1, USART2, USART3},
    PortPeripheral,
};
use orbit_common_proc_macro::define_ports;

use crate::port::ConfigureGPIO;

define_ports!(USART1 = 53, UART4 = 66);

#[derive(Clone, Copy)]
pub(crate) enum PortKinds {
    USART1,
    USART2,
    USART3,
    UART4,
}

impl ConfigureGPIO for PortKinds {
    fn configure(&self) {
        match self {
            PortKinds::USART1 => {
                // PA10 RX as Floating input
                // PA9 TX as push-pull alternate output
                let gpio = unsafe { &*chip::pac::GPIOA::PTR };
                unsafe {
                    gpio.cfghr
                        .modify(|r, w| w.bits(r.bits() | 0b1011 << 4 | 0b1000 << 8));
                    gpio.outdr.write(|w| w.bits(1 << 10));
                };
            }
            PortKinds::USART2 => {}
            PortKinds::USART3 => {}
            PortKinds::UART4 => {
                // PC11 RX as Floating input
                // PC10 TX as push-pull alternate output
                let gpio = unsafe { &*chip::pac::GPIOC::PTR };
                unsafe {
                    gpio.cfghr
                        .modify(|r, w| w.bits(r.bits() | 0b1011 << 8 | 0b1000 << 12));
                    gpio.outdr.write(|w| w.bits(1 << 11));
                };
            }
        }
    }
}
