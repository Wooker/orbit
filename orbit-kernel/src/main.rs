#![no_std]
#![no_main]

use orbit_arch;
use orbit_kernel::kernel::Kernel;

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
use orbit_arch::entry;

#[cfg(feature = "esp32c3")]
#[entry]
fn kernel_main() -> ! {
    let _peripherals = unsafe { KERNEL.initialize() };
    let a = 2;
    let b = 3;
    let mut c = a + b;
    loop {
        if c > 5 {
            continue;
        } else {
            c += 1;
        }
    }
}
