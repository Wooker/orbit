pub use ch59x::{Reg, RegisterSpec, ch59x as pac};

pub mod hil;

pub enum ClaimablePeripheral {
    UART1,
    I2C,
    GPIO,
}
pub trait Claimable {}
impl Claimable for pac::UART1 {}
impl Claimable for pac::I2C {}
impl Claimable for pac::GPIO {}
