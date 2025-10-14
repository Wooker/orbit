#![no_std]
#![no_main]

use orbit_kernel as _;

#[unsafe(no_mangle)]
pub extern "C" fn main() -> usize {
    let a = 0;
    let b = 1;
    a + b
}
