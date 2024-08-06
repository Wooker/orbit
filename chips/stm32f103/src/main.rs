#![no_std]
#![no_main]

use orbit::{entry, CPeripherals, DPeripherals, GpioExt, Kernel};

#[entry]
fn main() -> ! {
    let kernel = Kernel::new();
    // println!("Kernel initialized.");

    let peripherals = DPeripherals::take().unwrap();
    let mut gpioc = peripherals.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    led.set_high();

    loop {}
}
