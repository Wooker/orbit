#![no_std]
#![no_main]
#![allow(unused)]

#[allow(unused_imports)]
use orbit_libos::panic_handler as _;

#[no_mangle]
fn main() {
    let a = 2;
    let b = 3;

    let _c = a + b;
}
