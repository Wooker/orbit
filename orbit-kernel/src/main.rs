#![no_std]
#![no_main]

#[cfg(feature = "ch32v208")]
use orbit_arch::arch::qingke::entry;

#[panic_handler]
pub fn panic_handler<'a, 'b>(info: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {
        orbit_kernel::arch::qingke::riscv::asm::wfi();
    }
}

#[entry]
fn main() -> ! {
    let a = 0;

    loop {}
}
