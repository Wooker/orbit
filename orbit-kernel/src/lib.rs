#![no_std]
#![no_main]
#![allow(elided_named_lifetimes)]
#![allow(static_mut_refs)]
#![feature(maybe_uninit_uninit_array)]
#![feature(naked_functions)]

use core::arch::{asm, global_asm};

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

global_asm!(
    "
    .section .init
    .global _start

_start:
    la sp, _stack_top

    j main
    "
);
