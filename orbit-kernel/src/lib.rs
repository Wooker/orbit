#![no_std]
#![no_main]

pub mod kernel;

pub use orbit_arch;

#[export_name = "DefaultHandler"]
pub fn default_handler() {
    loop {}
}

// #[no_mangle]
// pub fn DefaultInterruptHandler() {
//     loop {}
// }

#[inline(never)]
#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {}
}
