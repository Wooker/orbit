pub use ch32v2::{Reg, RegisterSpec, ch32v20x as pac};

pub use ch32v2::ch32v20x::uart4::RegisterBlock as PortPeripheral;
pub const PORT_PTR: *const PortPeripheral = ch32v2::ch32v20x::UART4::PTR;

fn peripheral_to_interrupt_number(addr: usize) -> Option<usize> {
    use pac::*;
    match addr {
        _ => None
    }
}
