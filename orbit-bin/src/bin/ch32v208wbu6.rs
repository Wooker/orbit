#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use core::arch::asm;

use orbit_app::{application::Application, blinky_v208::Blinky, uart_v208::UartApp};
use orbit_kernel::{arch::entry, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel<4>;
    static mut BLINKY: Blinky;
    static mut UART_APP: UartApp<'static>;
}

#[entry]
unsafe fn main() -> ! {
    UART_APP.init("");
    KERNEL.add_application(0, UartApp::main as usize);
    KERNEL.initialize();
    loop {}
}
