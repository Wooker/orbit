#![no_std]
#![no_main]

use orbit::{entry, Kernel, Peripherals};

#[entry]
fn main() -> ! {
    let kernel = Kernel::new();
    // println!("Kernel initialized.");

    let peripherals = Peripherals::take().unwrap();
    let gpio = peripherals.GPIOC.split();
    // let io_mux = peripherals.IO_MUX;
    // let io = IO::new(gpio, io_mux);

    // let mac = efuse::Efuse::get_mac_address();
    // println!(
    //     "MAC: {:#X}:{:#X}:{:#X}:{:#X}:{:#X}:{:#X}",
    //     mac[0], mac[1], mac[2], mac[3], mac[4], mac[5]
    // );

    // println!("Available resources:\n{:?}", kernel.resources);

    // kernel.run(a);
    loop {}
}
