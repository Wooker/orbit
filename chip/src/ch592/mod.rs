pub use ch59x::{Reg, RegisterSpec, ch59x as pac};
pub mod hil;

use crate::claimable::Claimable;
impl Claimable for pac::UART1 {}
impl Claimable for pac::I2C {}
impl Claimable for pac::GPIO {}
