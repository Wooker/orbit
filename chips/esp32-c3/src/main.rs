#![no_std]
#![no_main]

use core::cell::RefCell;

use esp_backtrace as _;
pub use esp_hal::peripherals::Peripherals;
use esp_hal::{clock::ClockControl, entry, peripherals::*};
use esp_println::println;
use orbit::kernel::Kernel;

pub use esp_hal::adc::ADC;
use orbit::efuse;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let kernel = Kernel::new(peripherals);
    println!("Kernel initialized.");

    let system: &SYSTEM = &kernel.get_resources().SYSTEM;
    // let clocks = ClockControl::max(.clock_control).freeze();

    let mac = efuse::Efuse::get_mac_address();
    println!(
        "MAC: {:#X}:{:#X}:{:#X}:{:#X}:{:#X}:{:#X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    // println!("Available resources:\n{:?}", kernel.resources);

    loop {}
}
