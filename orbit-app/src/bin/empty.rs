#![no_std]
#![no_main]
#![allow(unused)]

// use orbit_kernel::kernel::KERNEL;
use orbit_kernel::panic_handler as _;

use core::panic::PanicInfo;

// extern "C" {
//     static KERNEL: Kernel;
// }

#[used]
static TEST: u8 = 1;

#[no_mangle]
fn main() -> ! {
    let mut a = 8;
    loop {
        a += TEST;
    }
}
