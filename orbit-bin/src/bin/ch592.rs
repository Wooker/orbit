#![no_std]
#![no_main]
#![allow(static_mut_refs)]
#![allow(unsafe_op_in_unsafe_fn)]
#![feature(integer_sign_cast)]
// #![forbid(unsafe_code)]

use core::arch::asm;

use orbit_app::{application::Application, blinky::Blinky, uart::UartApp};
use orbit_kernel::{arch::entry, kernel::Kernel};

unsafe extern "Rust" {
    static mut KERNEL: Kernel;
    static mut BLINKY: Blinky;
    static mut UART_APP: UartApp<'static>;
}

unsafe extern "C" {
    static _sapps: usize;
    static _eapps: usize;
}

fn _read_apps_section() -> &'static [usize] {
    unsafe {
        let start = &_sapps as *const usize;
        let end = &_eapps as *const usize;
        core::slice::from_raw_parts(
            start as *const usize,
            end.offset_from(start).cast_unsigned(),
        )
    }
}

#[entry]
unsafe fn main() -> ! {
    let a = _read_apps_section();
    asm!("nop");

    KERNEL.initialize();
    // UART_APP.init("SHT");
    // UART_APP.main();

    BLINKY.main();
    // KERNEL.schedule();
    loop {}
}
