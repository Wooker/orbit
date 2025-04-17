pub use ch32x0::{Reg, RegisterSpec, ch32x035 as pac};

pub use ch32x0::ch32x035::usart1::RegisterBlock as PortPeripheral;
pub const PORT_PTR: *const PortPeripheral = ch32x0::ch32x035::USART1::PTR;

use pac::*;
fn peripheral_to_interrupt_number(addr: usize) -> Option<usize> {
    match addr {
        _ => None
    }
}
