#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]
#![feature(ascii_char)]
#![feature(concat_bytes)]
#![feature(fn_align)]
#![feature(naked_functions_rustic_abi)]

use core::arch::{asm, global_asm};

pub mod application;
pub mod claim;
pub mod clock;
pub mod kernel;
pub mod port;
pub mod syscall;
pub mod task;
pub mod usizebuf;

pub use chip;
pub use orbit_arch as arch;

#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {}
}

#[inline(never)]
#[no_mangle]
#[link_section = "interrupt_handler.uart4"]
unsafe extern "C" fn UART4() {
    loop {
        asm!("nop")
    }
}

global_asm!(
    "
    .section .init
    .global _start

_start:
    la sp, _stack_top
    ",
    // "
    // li t0, 0x1f
    // csrw 0xbc0, t0
    // li t0, 0x08
    // csrw 0x804, t0
    // ",
    "li t0, 0x1880",
    "csrw mstatus, t0",
    "la ra, initialize_finish",
    // Set mtvec
    "
    la t0, handler;
    csrw mtvec, t0;
    ",
    // Set dcsr 9 and 11 bits
    "
    csrr t0, dcsr;
    li t1, 0xa00;
    or t0, t0, t1;
    csrw dcsr, t0;
    ",
    "
    // la t0, main;
    // csrw mepc, t0;
    // mret;
    j main;
    "
);
