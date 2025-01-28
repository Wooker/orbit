pub use ch32v2::{Reg, RegisterSpec, ch32v20x as pac};

use crate::claimable::Claimable;
impl Claimable for pac::UART4 {}
impl Claimable for pac::I2C2 {}
impl Claimable for pac::GPIOA {}
impl Claimable for pac::GPIOB {}
impl Claimable for pac::RCC {}
