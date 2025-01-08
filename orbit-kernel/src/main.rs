#![no_std]
#![no_main]

use core::arch::global_asm;

#[cfg(feature = "ch32v208")]
// use orbit_arch::arch::qingke::entry;
#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {
        orbit_kernel::arch::qingke::riscv::asm::wfi();
    }
}

global_asm!(
    "
    .global _start;
    
    _start:
        csrwi mie, 0;
        j kernel_main;
"
);

#[allow(unused)]
// #[entry]
#[no_mangle]
fn kernel_main() -> ! {
    let a = 0;

    loop {}
}
