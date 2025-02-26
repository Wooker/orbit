#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky_v208::Blinky, uart_v208::UartApp};
use orbit_kernel::kernel::Kernel;

unsafe extern "Rust" {
    static mut KERNEL: Kernel<4>;
    static mut BLINKY: Blinky;
    static mut UART_APP: UartApp;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".bin.text")]
unsafe fn main() -> ! {
    BLINKY.init();
    UART_APP.init();
    KERNEL.add_application(
        0,
        unsafe { &UART_APP as *const UartApp as usize },
        UartApp::main as usize,
        UartApp::stack_top(),
    );
    KERNEL.add_application(
        1,
        unsafe { &BLINKY as *const Blinky as usize },
        Blinky::main as usize,
        0x00,
    );
    KERNEL.initialize();
    loop {}
}
