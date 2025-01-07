#![no_std]
#![no_main]

#[allow(unused_imports)]
use orbit_libos::panic_handler as _;

#[no_mangle]
fn main() {
    let _a = 2;
}
