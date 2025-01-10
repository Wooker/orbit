#![no_std]
#![no_main]

use core::arch::global_asm;

#[cfg(feature = "ch32v208")]
// use orbit_arch::arch::qingke::entry;

global_asm!(
    "
    .global _start;
    
    _start:
        csrwi mie, 0;
        j kernel_main;
"
);

use orbit_kernel::kernel::KERNEL;

#[allow(unused)]
#[no_mangle]
fn kernel_main() -> ! {
    let p = KERNEL.initialize();
    unsafe {
        p.GPIO.pa_pd_drv.modify(|_, w| w.bits(1 << 4));
        p.GPIO.pa_dir.modify(|_, w| w.bits(1 << 4));

        p.GPIO.pb_pd_drv.modify(|_, w| w.bits(1 << 23));
        p.GPIO.pb_dir.modify(|_, w| w.bits(1 << 23));

        p.GPIO.pb_out.modify(|r, w| w.bits(r.bits() ^ (1 << 23)));
    }

    loop {
        // unsafe {
        //     p.GPIOA
        //         .pa_out
        //         .modify(|r, w| w.pa_out().bits(r.pa_out().bits() ^ (1 << 4)));
        //     p.GPIOA
        //         .pb_out
        //         .modify(|r, w| w.pb_out().bits(r.pb_out().bits() ^ (1 << 23)));
        //     riscv::asm::delay(100000);
        // }
    }
}
