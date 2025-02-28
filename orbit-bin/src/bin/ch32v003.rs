#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use core::arch::asm;

use orbit_app::{application::Application, blinky_v003::Blinky, uart_v003::UartApp};
use orbit_kernel::kernel::Kernel;

unsafe extern "Rust" {
    static mut KERNEL: Kernel<0>;
    static mut BLINKY: Blinky;
    static mut UART_APP: UartApp;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bin.text")]
unsafe fn main() -> ! {
    BLINKY.init();
    KERNEL.add_application(
        0,
        unsafe { &UART_APP as *const UartApp as usize },
        UartApp::main as usize,
        UartApp::stack_top(),
    );
    KERNEL.add_application(
        1,
        unsafe { &UART_APP as *const UartApp as usize },
        Blinky::main as usize,
        Blinky::stack_top(),
    );
    KERNEL.initialize();
    loop {}
}
