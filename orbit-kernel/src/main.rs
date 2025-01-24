#![no_std]
#![no_main]

use orbit_arch;
use orbit_arch::entry;
use orbit_arch::interface::{pmp::Pmp, timer::Timer};
use orbit_kernel::kernel::Kernel;

use chip::*;

#[cfg(feature = "ch592")]
#[allow(unused)]
#[no_mangle]
#[entry]
fn kernel_main() -> ! {
    let mut kernel = Kernel::new(32000);
    kernel.core.pmp.clear_cfg(0, 0);
    let p = &kernel.peripherals;
    let a = kernel.claim::<chip::pac::adc::adc_cfg::ADC_CFG_SPEC>();
    let mut pa8: PA8 = GPIO::new();
    pa8.enable();
    pa8.set_high();
    loop {
        pa8.set_high();
        kernel.core.timer.delay(480000000);
        // // orbit_arch::riscv::asm::delay(10000);
        pa8.set_low();
        // kernel.core.timer.delay(480000);
        // // orbit_arch::riscv::asm::delay(10000);
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
