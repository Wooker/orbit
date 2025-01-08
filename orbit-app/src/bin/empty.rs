#![no_std]
#![no_main]
#![allow(unused)]

use orbit_kernel::panic_handler as _;

#[no_mangle]
fn main() {
    let a = 2;
    let b = 3;

    let _c = a + b;
}
