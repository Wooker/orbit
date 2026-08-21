// pub use ch32x0::{Reg, RegisterSpec, ch32x035 as pac};
// pub use pac::usart1::RegisterBlock as PortPeripheral;
pub const RAM_SIZE: usize = 32 * 1024;
pub use pac::PortPeripheral;

pub mod pac {
    pub struct PortPeripheral;
    pub struct UART;
    impl UART {
        pub const PTR: *const PortPeripheral = core::ptr::null();
    }
}
