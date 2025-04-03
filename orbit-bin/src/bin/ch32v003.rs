#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use core::arch::asm;

use orbit_app::{application::Application, blinky_v003::Blinky};
use orbit_kernel::{claim::KernelPeripherals, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel<'static>;
    static mut BLINKY: Blinky;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bin.text")]
unsafe fn main() -> ! {
    KERNEL.clock.freeze();

    BLINKY.init();
    KERNEL.add_application(
        1,
        unsafe { &BLINKY as *const Blinky as usize },
        Blinky::main as usize,
        None,
        BLINKY.context(),
        [],
    );
    KERNEL.initialize();
    panic!();
}
