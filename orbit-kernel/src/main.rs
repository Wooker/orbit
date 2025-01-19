#![no_std]
#![no_main]

use core::arch::global_asm;

use orbit_kernel::kernel::Kernel;

global_asm!(
    "
    .global start;
start:
    csrwi mstatus, 0;   
"
);

#[cfg(feature = "ch592")]
use orbit_arch::entry;

#[cfg(feature = "ch592")]
#[allow(unused)]
#[no_mangle]
#[entry]
fn kernel_main() -> ! {
    let p = KERNEL.initialize();
    unsafe {
        // Set PA8 as output with 20mA level
        p.GPIO.pa_pd_drv.modify(|_, w| w.bits(1 << 8));
        // Set PB23 as output with 20mA level
        p.GPIO.pb_pd_drv.modify(|_, w| w.bits(1 << 23));
        p.GPIO.pb_dir.modify(|_, w| w.bits(1 << 23));

        p.GPIO.pa_out.modify(|r, w| w.bits(r.bits() ^ (1 << 8)));
        p.GPIO.pb_out.modify(|r, w| w.bits(r.bits() ^ (1 << 23)));
    }

    loop {
        unsafe {
            p.GPIO.pa_out.modify(|r, w| w.bits(r.bits() ^ (1 << 8)));
            p.GPIO.pb_out.modify(|r, w| w.bits(r.bits() ^ (1 << 23)));
            orbit_arch::qingke::riscv::asm::delay(10000);

            p.GPIO.pa_out.modify(|r, w| w.bits(r.bits() ^ (0 << 8)));
            p.GPIO.pb_out.modify(|r, w| w.bits(r.bits() ^ (0 << 23)));
            orbit_arch::qingke::riscv::asm::delay(10000);
        }
    }
}

#[cfg(feature = "ch32v208wbu6")]
use chip::Peripherals;
#[cfg(feature = "ch32v208wbu6")]
use orbit_arch::entry;

extern "C" {
    static mut DEVICE_PERIPHERALS: bool;
    static mut KERNEL: Kernel;
}

#[cfg(feature = "ch32v208wbu6")]
#[allow(unused)]
#[no_mangle]
#[entry]
unsafe fn kernel_main() -> ! {
    KERNEL.initialize();
    let gpiob = KERNEL.claim();
    // Set PB8 as output with 50Mhz speed
    (*gpiob).cfghr.modify(|_, w| w.bits(0b0101));
    // Reset PB8
    (*gpiob).bshr.write(|w| w.bits(1 << 24));

    loop {
        unsafe {
            (*gpiob).bshr.write(|w| w.bits(1 << 8));
            orbit_arch::qingke::riscv::asm::delay(1000000);

            (*gpiob).bshr.write(|w| w.bits(1 << 24));
            orbit_arch::qingke::riscv::asm::delay(1000000);
        }
    }
}

// #[cfg(feature = "esp32c3")]
// use orbit_arch::entry;

#[cfg(feature = "esp32c3")]
#[link_section = ".trap.rust"]
fn DefaultHandler() {}

#[cfg(feature = "esp32c3")]
#[no_mangle]
fn kernel_main() -> ! {
    let peripherals = unsafe { KERNEL.initialize() };
    loop {
        let a = 2;
        let b = 3;
        let c = a + b;
    }

    // // Get access to the GPIO registers
    // let gpio = &peripherals.GPIO;

    // // Configure GPIO2 as an output (replace with your LED's GPIO pin number)
    // const LED_GPIO: u8 = 8;

    // unsafe {
    //     // Disable the GPIO function to ensure it's in the default state
    //     gpio.enable_w1tc().write(|w| w.bits(1 << LED_GPIO));

    //     // Set GPIO2 to output mode
    //     gpio.func_out_sel_cfg(LED_GPIO as usize).modify(
    //         |_, w| w.out_sel().bits(255), // Connect the GPIO to the GPIO output signal
    //     );

    //     gpio.enable_w1ts().write(|w| w.bits(1 << LED_GPIO));
    // }
    // loop {
    //     unsafe {
    //         // Set the LED pin high (turn the LED on)
    //         gpio.out_w1ts().write(|w| w.bits(1 << LED_GPIO));

    //         // Delay manually to control timing
    //         for _ in 0..1_000_000 {
    //             core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    //         }

    //         // Set the LED pin low (turn the LED off)
    //         gpio.out_w1tc().write(|w| w.bits(1 << LED_GPIO));

    //         // Delay again
    //         for _ in 0..1_000_000 {
    //             core::sync::atomic::compiler_fence(core::sync::atomic::Ordering::SeqCst);
    //         }
    //     }
    // }
}
