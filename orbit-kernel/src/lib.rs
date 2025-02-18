#![no_std]
#![no_main]
#![allow(elided_named_lifetimes)]
#![allow(static_mut_refs)]
#![feature(maybe_uninit_uninit_array)]

use core::arch::asm;

pub mod application;
pub mod claim;
pub mod clock;
pub mod kernel;
pub mod task;

pub use chip;
pub use orbit_arch as arch;

#[panic_handler]
pub fn panic_handler<'a, 'b>(info: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {}
}
