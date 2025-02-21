#![no_std]
#![no_main]

use orbit_common::{feature_mod, feature_mod_use};

pub mod claimable;
pub mod interface;

#[cfg(feature = "esp32c3")]
pub mod esp32c3;
#[cfg(feature = "esp32c3")]
pub use esp32c3::*;

#[cfg(feature = "ch32v208wbu6")]
pub mod ch32v208wbu6;
#[cfg(feature = "ch32v208wbu6")]
pub use ch32v208wbu6::*;

#[cfg(feature = "ch592")]
pub mod ch592;
#[cfg(feature = "ch592")]
pub use ch592::hil;
// #[cfg(feature = "ch592")]
// pub use ch592::hil::*;
#[cfg(feature = "ch592")]
pub use ch592::*;

feature_mod_use!("bl702", pub);

feature_mod_use!("ch32v003", pub);
