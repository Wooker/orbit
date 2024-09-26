#![no_std]
#![no_main]

use orbit::{esp_riscv_rt::entry, Kernel};

#[entry]
fn main() -> ! {
    // let kernel = Kernel::new();
    // println!("Kernel initialized.");

    // let peripherals = Peripherals::take();
    // let gpio = peripherals.GPIO;
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

#[no_mangle]
extern "C" fn EspDefaultHandler() {
    #[cfg(not(feature = "defmt"))]
    panic!("Unhandled interrupt: ");

    #[cfg(feature = "defmt")]
    panic!("Unhandled interrupt: {:?}", defmt::Debug2Format());
}
