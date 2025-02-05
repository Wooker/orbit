#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky::Blinky, uart::UartApp};
use orbit_kernel::{arch::entry, kernel::Kernel, kernel::Rate};

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
    static mut BLINKY: Blinky;
    static mut UART_APP: UartApp;
}

#[entry]
unsafe fn main() -> ! {
    KERNEL.initialize(Rate::<u32, 1, 1>::MHz(60));
    UART_APP.main();
    BLINKY.main();
    // KERNEL.schedule();
    loop {}
}
