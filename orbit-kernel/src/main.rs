#![no_std]
#![no_main]

use orbit_arch;
use orbit_arch::entry;
use orbit_kernel::kernel::Kernel;

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
#[entry]
fn kernel_main() -> ! {
    let kernel = Kernel::new();
    let (_maj, _min) = kernel.version();

    kernel.initialize();
}

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
