pub use ch59x::{Reg, RegisterSpec, ch59x as pac};

pub use ch59x::ch59x::uart0::RegisterBlock as PortPeripheral;
pub const PORT_PTR: *const PortPeripheral = ch59x::ch59x::UART0::PTR;
