#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use orbit_arch;
use orbit_arch::entry;
use orbit_arch::interface::{pmp::Pmp, timer::Timer};
use orbit_kernel::kernel::{claim::Claim, Kernel};

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
#[entry]
fn kernel_main() -> ! {
    use chip::pac::{GPIO as PACGPIO, I2C, UART1};
    use orbit_kernel::kernel::claim::Claimed;

    unsafe { KERNEL.initialize() };
    unsafe { KERNEL.core.pmp.clear_cfg(0, 0) };

    let mut uart: Claimed<UART1> = unsafe {
        KERNEL
            .claim(chip::ClaimablePeripheral::UART1)
            .unwrap_unchecked()
    };

    let mut i2c: Claimed<I2C> = unsafe {
        KERNEL
            .claim(chip::ClaimablePeripheral::I2C)
            .unwrap_unchecked()
    };
    i2c.modify(|p| {
        p.i2c_ctrl1.modify(|r, w| unsafe { w.bits(r.bits()) });
        p.i2c_ctrl2.modify(|r, w| unsafe { w.bits(r.bits()) });
    });
    i2c.revoke();
    uart.revoke();

    let mut gpio: Claimed<PACGPIO> = unsafe {
        KERNEL
            .claim(chip::ClaimablePeripheral::GPIO)
            .unwrap_unchecked()
    };
    gpio.modify(|p| {
        p.pa_dir.write(|w| unsafe { w.bits(1 << 8) });
    });

    unsafe { KERNEL.core.timer.delay(KERNEL_MINOR as u32 * 200000) };
    gpio.modify(|p| {
        p.pa_dir
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 8)) });
    });

    // let mut pa8: PA8 = GPIO::new();
    // pa8.enable();
    loop {
        // pa8.set_high();
        // unsafe { KERNEL.core.timer.delay(KERNEL_MINOR as u32 * 200000) };
        // pa8.set_low();
        // unsafe { KERNEL.core.timer.delay(200000 + KERNEL_MAJOR as u32) };
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
