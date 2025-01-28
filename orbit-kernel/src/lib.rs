#![no_std]
#![no_main]
#![allow(elided_named_lifetimes)]

pub mod kernel;
pub mod task;

pub use orbit_arch;

#[export_name = "DefaultHandler"]
pub fn default_handler() {
    loop {}
}

#[inline(never)]
#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {}
}
