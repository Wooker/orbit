#![no_std]
#![no_main]

pub mod kernel;

use crate::kernel::Kernel;
pub use orbit_arch;

// extern "C" {
//     static mut KERNEL: Kernel;
// }

#[no_mangle]
pub fn DefaultHandler() {
    loop {}
}

// #[no_mangle]
// pub fn DefaultInterruptHandler() {
//     loop {}
// }

#[inline(never)]
#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    // unsafe {
    //     KERNEL.initialize();
    // }
    // loop {
    //     unsafe {
    //         if let Some(ref p) = KERNEL.peripherals {
    //             // arch::qingke::riscv::asm::wfi();
    //             p.GPIOB.bshr.write(|w| w.bits(1 << 8));
    //             orbit_arch::qingke::riscv::asm::delay(1000000);

    //             p.GPIOB.bshr.write(|w| w.bits(1 << 24));
    //             orbit_arch::qingke::riscv::asm::delay(1000000);
    //         }
    //     }
    // }
    loop {}
}
