#![no_std]
#![no_main]

use core::arch::global_asm;

use orbit_kernel::kernel::Kernel;

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
    let gpiob = &p.GPIOB;
    gpiob.cfghr.modify(|r, w| unsafe { w.bits(r.bits() | 3) });
    gpiob
        .outdr
        .modify(|r, w| unsafe { w.bits(r.bits() | 0x0ff) });

    loop {}
}
