#![no_std]
#![no_main]

use core::cell::RefCell;

use esp_backtrace as _;
use esp_hal::{entry, peripheral, peripherals::Peripherals};
use esp_println::println;
use kernel::Resource;

use crate::kernel::Kernel;

mod kernel;

pub use esp_hal::adc::ADC;

impl Resource for Peripherals {}

#[entry]
fn main() -> ! {
    let peripherals = Peripherals::take();

    let kernel = Kernel::new(peripherals);
    let dma = kernel.resources.DMA;
    println!("Kernel initialized.");
    let adc = kernel.resources.ADC1;
    // println!("Available resources:\n{:?}", kernel.resources);

    loop {}
}
