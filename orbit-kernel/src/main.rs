#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use orbit_arch;
use orbit_arch::entry;
use orbit_arch::interface::{pmp::Pmp, timer::Timer};
use orbit_kernel::kernel::Kernel;

use chip::{
    hil::gpio::{GPIO, PA8},
    interface::gpio::OrbitGPIO,
};

extern "Rust" {
    static mut KERNEL: Kernel;
    static KERNEL_MINOR: u8;
    static KERNEL_MAJOR: u8;
}

#[cfg(feature = "ch592")]
#[allow(unused)]
#[no_mangle]
#[entry]
fn kernel_main() -> ! {
    unsafe { KERNEL.initialize() };
    unsafe { KERNEL.core.pmp.clear_cfg(0, 0) };
    let mut pa8: PA8 = GPIO::new();
    pa8.enable();
    loop {
        pa8.set_high();
        unsafe { KERNEL.core.timer.delay(KERNEL_MINOR as u32 * 100000) };
        pa8.set_low();
        unsafe { KERNEL.core.timer.delay(100000 + KERNEL_MAJOR as u32) };
    }
}

#[cfg(feature = "ch32v208wbu6")]
#[entry]
fn kernel_main() -> ! {
    let kernel = Kernel::new();
    let (_maj, _min) = kernel.version();

    kernel.initialize();
}

#[cfg(feature = "esp32c3")]
#[no_mangle]
// #[entry]
fn hal_main() -> ! {
    let kernel = Kernel::new(40_000_000);
    let p = &kernel.peripherals;
    let gpio = &p.GPIO;
    gpio.func8_out_sel_cfg().write(|w| unsafe { w.bits(0x80) });
    gpio.out_w1ts().write(|w| unsafe { w.bits(1 << 8) });

    loop {}
}
