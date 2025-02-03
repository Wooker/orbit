#![no_std]
#![no_main]
#![allow(elided_named_lifetimes)]
#![allow(static_mut_refs)]

pub mod kernel;
pub mod task;

pub use chip;
pub use orbit_arch as arch;

#[export_name = "DefaultHandler"]
pub fn default_handler() {
    loop {}
}

#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {}
}
