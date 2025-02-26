#![no_std]
#![no_main]

use core::panic::PanicInfo;

#[used]
static BUF: u8 = 2;

#[inline(never)]
unsafe fn add(a: u8, b: u8) -> u8 {
    a.unchecked_add(b)
}

#[no_mangle]
unsafe fn main() -> ! {
    let a = BUF;
    let b = 3;
    let mut c = add(a, b);
    loop {
        c += 1;
        if c == 100 {
            break;
        } else {
            continue;
        }
    }
    loop {}
}

#[panic_handler]
fn panic(_: &PanicInfo) -> ! {
    loop {}
}
