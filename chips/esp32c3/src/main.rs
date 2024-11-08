#![no_std]
#![no_main]

use orbit::{
    esp_backtrace as _,
    hal::{efuse, entry, gpio::IO, peripherals::Peripherals},
    kernel::Kernel,
    println::println,
};

#[entry]
#[no_mangle]
fn main() -> ! {
    let kernel = Kernel::new();
    println!("Kernel initialized.");

    let peripherals = Peripherals::take();
    let gpio = peripherals.GPIO;
    let io_mux = peripherals.IO_MUX;
    let io = IO::new(gpio, io_mux);

    let mac = efuse::Efuse::get_mac_address();
    println!(
        "MAC: {:#X}:{:#X}:{:#X}:{:#X}:{:#X}:{:#X}",
        mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    );

    // println!("Available resources:\n{:?}", kernel.resources);

    kernel.run(a);
}

fn a() -> i32 {
    42
}
