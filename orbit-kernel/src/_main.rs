#![no_std]
#![no_main]
#![allow(static_mut_refs)]

use orbit_arch;
use orbit_arch::entry;
use orbit_kernel::kernel::{claim::Claim, Kernel};

extern "Rust" {
    static mut KERNEL: Kernel;
    static _KERNEL_MINOR: u8;
    static _KERNEL_MAJOR: u8;
}

extern "Rust" {
    static APPS: [u8; 4];
}

#[cfg(feature = "ch592")]
#[entry]
fn kernel_main() -> ! {
    use chip::pac::GPIO as PACGPIO;
    use orbit_arch::interface::timer::Timer;
    use orbit_kernel::kernel::claim::Claimed;

    unsafe { KERNEL.initialize() };
    // let mut gpio: Claimed<PACGPIO> = unsafe { KERNEL.claim().unwrap_unchecked() };

    // loop {
    //     unsafe { KERNEL.core.timer.delay(200000) };
    //     gpio.modify(|p| {
    //         p.pa_dir
    //             .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 8)) });
    //     });
    //     unsafe { KERNEL.core.timer.delay(200000) };
    //     gpio.modify(|p| {
    //         p.pa_dir.write(|w| unsafe { w.bits(1 << 8) });
    //     });
    // }
}

#[cfg(feature = "ch32v208wbu6")]
#[entry]
fn kernel_main() -> ! {
    use chip::pac::{GPIOB, RCC};
    use orbit_arch::interface::timer::Timer;
    use orbit_kernel::kernel::claim::Claimed;

    unsafe { KERNEL.initialize() };

    let apb2_bits = (1 << 3) + (1 << 14); // PB, USART1
    let mut rcc: Claimed<RCC> = unsafe { KERNEL.claim().unwrap_unchecked() };
    let mut gpiob: Claimed<GPIOB> = unsafe { KERNEL.claim().unwrap_unchecked() };
    rcc.modify(|p| {
        p.apb2prstr.write(|w| unsafe { w.bits(apb2_bits) });
        p.apb2prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(apb2_bits)) });
        p.apb2pcenr.write(|w| unsafe { w.bits(apb2_bits) });
    });

    gpiob.modify(|p| {
        p.cfghr.write(|w| unsafe { w.bits(0b0010) });
        p.bshr.write(|w| unsafe { w.bits(1 << 8) });
    });
    loop {
        unsafe { KERNEL.core.timer.delay(200000) };
        gpiob.modify(|p| {
            p.bshr.write(|w| unsafe { w.bits(1 << 8) });
        });
        unsafe { KERNEL.core.timer.delay(200000) };
        gpiob.modify(|p| {
            p.bshr.write(|w| unsafe { w.bits(!(1 << (8 + 15))) });
        });
    }
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
