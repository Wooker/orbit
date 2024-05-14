#![no_std]
#![no_main]

use orbit::{
    esp_backtrace as _,
    hal::{
        clock::ClockControl,
        efuse, entry,
        peripherals::{AES, SYSTEM},
        prelude::*,
        Delay,
    },
    kernel::Kernel,
    println::println,
};

#[entry]
#[no_mangle]
fn main() -> ! {
    let kernel = Kernel::new();
    println!("Kernel initialized.");

    let system = kernel.multiplexer.system().split();
    let clocks = ClockControl::max(system.clock_control).freeze();
    let delay = Delay::new(&clocks);

    let aes = kernel.multiplexer.aes();
    let dma = kernel.multiplexer.dma();
    let aes2 = kernel.multiplexer.aes();

    let mac = efuse::Efuse::get_mac_address();
    println!(
        "MAC: {:#X}:{:#X}:{:#X}:{:#X}:{:#X}:{:#X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    // println!("Available resources:\n{:?}", kernel.resources);

    loop {
        println!("In a loop");
        delay.delay_micros(1000 * 1000);
    }
}
