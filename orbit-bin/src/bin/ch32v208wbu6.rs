#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
// #![forbid(unsafe_code)]

use orbit_app::{application::Application, blinky_v208::Blinky, uart_v208::UartApp, wfi::Wfi};
use orbit_kernel::{claim::KernelPeripherals, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel<'static>;
    static mut WFI: Wfi;
    static mut BLINKY: Blinky;
    static mut UART_APP: UartApp<'static>;
}

#[unsafe(no_mangle)]
#[unsafe(link_section = ".text.bin")]
unsafe fn main() -> ! {
    KERNEL.clock.freeze();

    WFI.init();
    UART_APP.init();

    KERNEL.add_application(
        0,
        unsafe { &WFI as *const Wfi as usize },
        Wfi::main as usize,
        None,
        WFI.context(),
        [None, None, None, None],
    );
    KERNEL.add_application(
        1,
        unsafe { &UART_APP as *const UartApp as usize },
        UartApp::main as usize,
        Some(UartApp::interrupt as usize),
        UART_APP.context(),
        [
            Some(KernelPeripherals::GPIOC),
            Some(KernelPeripherals::UART4),
            None,
            None,
        ],
    );
    BLINKY.init();
    KERNEL.add_application(
        2,
        unsafe { &BLINKY as *const Blinky as usize },
        Blinky::main as usize,
        Some(Blinky::interrupt as usize),
        BLINKY.context(),
        [None, None, None, None],
    );

    KERNEL.initialize();
    panic!();
}
