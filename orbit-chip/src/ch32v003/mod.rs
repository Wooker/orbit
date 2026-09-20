pub use ch32v0::{Reg, RegisterSpec, ch32v003 as pac};

pub use ch32v0::ch32v003::usart1::RegisterBlock as PortPeripheral;
pub const PORT_PTR: *const PortPeripheral = ch32v0::ch32v003::USART1::PTR;
