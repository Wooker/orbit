#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky_v208::Blinky, wfi::Wfi};
use orbit_kernel::{claim::KernelPeripherals, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel<'static>;
    static mut WFI: Wfi;
    static mut BLINKY: Blinky;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.bin")]
unsafe fn main() -> ! {
    KERNEL.clock.freeze();

    BLINKY.init();
    KERNEL.add_application(
        0,
        unsafe { &BLINKY as *const Blinky as usize },
        Blinky::main as usize,
        Some(Blinky::interrupt as usize),
        BLINKY.context(),
        [Some(KernelPeripherals::GPIOB), None, None, None],
    );
    WFI.init();
    KERNEL.add_application(
        1,
        unsafe { &WFI as *const Wfi as usize },
        Wfi::main as usize,
        Some(Wfi::interrupt as usize),
        WFI.context(),
        [None, None, None, None],
    );

    KERNEL.initialize();
    panic!();
}
