#![no_std]
#![no_main]

use core::cell::RefCell;

use esp_backtrace as _;
pub use esp_hal::peripherals::Peripherals;
use esp_hal::{
    clock::ClockControl,
    entry, peripheral,
    peripherals::{DMA, SYSTEM},
};
use esp_println::println;
use orbit::kernel::{Kernel, Resource};

pub use esp_hal::adc::ADC;

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();
    let kernel = Kernel::new(peripherals);
    println!("Kernel initialized.");

    let system: &SYSTEM = &kernel.get_resources().SYSTEM;
    // let clocks = ClockControl::max(system.clock_control).freeze();

    let mac = esp_hal::efuse::Efuse::get_mac_address();
    println!(
        "MAC: {:#X}:{:#X}:{:#X}:{:#X}:{:#X}:{:#X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    // println!("Available resources:\n{:?}", kernel.resources);

    loop {}
}
