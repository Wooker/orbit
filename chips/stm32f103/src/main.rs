#![no_std]
#![no_main]

use core::arch::asm;
use orbit::{entry, CPeripherals, FlashExt, GpioExt, Kernel, Peripherals, RccExt, Timer, *};

#[entry]
fn main() -> ! {
    // let kernel = Kernel::new();
    // hprintln!("Kernel initialized.");

    let dp = Peripherals::take().unwrap();
    let cp = CPeripherals::take().unwrap();

    let mut flash = dp.FLASH.constrain();
    let rcc = dp.RCC.constrain();

    let clocks = rcc.cfgr.freeze(&mut flash.acr);

    let mut gpioc = dp.GPIOC.split();
    let mut led = gpioc.pc13.into_push_pull_output(&mut gpioc.crh);

    let mut timer = Timer::syst(cp.SYST, &clocks).counter_hz();
    timer.start(1.Hz()).unwrap();

    loop {
        timer.wait().unwrap();
        led.set_high();
        timer.wait().unwrap();
        led.set_low();
    }
}
