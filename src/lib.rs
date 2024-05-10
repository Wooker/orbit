//! Orbit

#![no_std]
#![no_main]

pub mod kernel;

mod claim;
pub mod multiplexer;
mod resource;
mod resource_manager;

/*
#[cfg(feature = "esp32-c3")]
pub use esp_backtrace;
#[cfg(feature = "esp32-c3")]
pub use esp_hal as hal;
#[cfg(feature = "esp32-c3")]
pub use esp_println as println;
#[cfg(feature = "esp32-c3")]
impl Resource for hal::peripherals::Peripherals {
    fn claim() -> Rc<Self> {
        let rf = Rc::new(Peripherals::take());
        let per = rf.
        rf
    }
}
*/

pub use esp_backtrace;
pub use esp_hal as hal;
pub use esp_println as println;
