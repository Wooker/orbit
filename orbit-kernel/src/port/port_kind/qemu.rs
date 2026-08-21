use chip::PortPeripheral;
use orbit_common_proc_macro::define_ports;

use crate::port::ConfigureGPIO;

define_ports!(UART = 0);

#[derive(Clone, Copy)]
pub(crate) enum PortKinds {
    UART,
}

impl ConfigureGPIO for PortKinds {
    fn configure(&self) {
        match self {
            _ => {}
        }
    }
}
