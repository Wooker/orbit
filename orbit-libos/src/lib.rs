#![no_std]
#![no_main]

use core::panic::PanicInfo;

pub use orbit_kernel::*;

#[cfg(feature = "ch32-hal")]
pub use ch32_hal::*;

#[panic_handler]
pub fn panic_handler<'a, 'b>(info: &'a PanicInfo<'b>) -> ! {
    loop {
        orbit_kernel::arch::qingke::riscv::asm::wfi();
    }
}
