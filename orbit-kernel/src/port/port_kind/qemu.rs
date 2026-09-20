use orbit_chip::{PortPeripheral, pac::UART0};
use orbit_common_proc_macro::define_ports;

use crate::port::ConfigureGPIO;

define_ports!(UART0 = 10);

#[derive(Clone, Copy)]
pub(crate) enum PortKinds {
    UART0,
}

impl ConfigureGPIO for PortKinds {
    fn configure(&self) {
        match self {
            _ => {}
        }
    }
}
