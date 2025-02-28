#![no_std]
#![no_main]
#![allow(elided_named_lifetimes)]
#![allow(static_mut_refs)]
#![feature(maybe_uninit_uninit_array)]
#![feature(naked_functions)]

use core::arch::{asm, global_asm};
use core::mem::MaybeUninit;

pub mod application;
pub mod claim;
pub mod clock;
pub mod kernel;
pub mod task;

pub use chip;
pub use orbit_arch as arch;

#[no_mangle]
fn DefaultHandler() {
    loop {}
}

#[panic_handler]
pub fn panic_handler<'a, 'b>(info: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {}
}

// #[cfg(feature = "pfic")]
// #[used]
// #[link_section = ".vector_table.interrupts"]
// pub(crate) static mut _VECTOR_TABLE_INTERRUPTS: [usize; 255] = [0x000003fc; 255];

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
    "
    li t0, 0x1f
    csrw 0xbc0, t0
    li t0, 0x08
    csrw 0x804, t0
    ",
    "li t0, 0x1880",
    "csrw mstatus, t0",
    "la t0, main",
    "csrw mepc, t0",
    "csrr t0, dcsr",
    "ori t0, t0, 0x200",
    "csrw dcsr, t0",
    "mret",
    // "jalr zero, t0, 0"
);
