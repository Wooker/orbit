#![no_std]
#![no_main]

use core::arch::global_asm;
use orbit_arch::arch;

// global_asm!(
//     "
//     .global _start;

//     _start:
//         csrwi mie, 0;
//         j kernel_main;
// "
// );

use orbit_kernel::kernel::KERNEL;

#[cfg(feature = "ch592")]
use orbit_arch::arch::qingke_rt::entry;

#[cfg(feature = "ch592")]
#[allow(unused)]
#[no_mangle]
#[entry]
fn kernel_main() -> ! {
    let p = KERNEL.initialize();
    unsafe {
        // Set PA8 as output with 20mA level
        p.GPIO.pa_pd_drv.modify(|_, w| w.bits(1 << 8));
        p.GPIO.pa_dir.modify(|_, w| w.bits(1 << 8));

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
            arch::qingke::riscv::asm::delay(100000);

            p.GPIO.pa_out.modify(|r, w| w.bits(r.bits() ^ (0 << 8)));
            p.GPIO.pb_out.modify(|r, w| w.bits(r.bits() ^ (0 << 23)));
            arch::qingke::riscv::asm::delay(100000);
        }
    }
}

#[cfg(feature = "ch32v208wbu6")]
use orbit_arch::arch::qingke_rt::entry;

#[cfg(feature = "ch32v208wbu6")]
#[allow(unused)]
#[no_mangle]
#[entry]
fn kernel_main() -> ! {
    let p = KERNEL.initialize();
    unsafe {
        // Reset GPIO PORT B
        p.RCC.apb2prstr.modify(|_, w| w.bits(1 << 3));
        p.RCC.apb2prstr.modify(|r, w| w.bits(r.bits() & !(1 << 3)));

        // Enable GPIO PORT B
        p.RCC.apb2pcenr.modify(|_, w| w.bits(1 << 3));

        // Set PB8 as output with 50Mhz speed
        p.GPIOB.cfghr.modify(|_, w| w.bits(0b0101));

        p.GPIOB.bshr.write(|w| w.bits(1 << 24));
    }

    loop {
        unsafe {
            p.GPIOB.bshr.write(|w| w.bits(1 << 8));
            arch::qingke::riscv::asm::delay(1000000);

            p.GPIOB.bshr.write(|w| w.bits(1 << 24));
            arch::qingke::riscv::asm::delay(1000000);
        }
    }
}

#[cfg(feature = "esp32c3")]
use arch::riscv_rt::entry;

#[cfg(feature = "esp32c3")]
#[entry]
fn kernel_main() -> ! {
    loop {}
    // let peripherals = KERNEL.initialize();

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
