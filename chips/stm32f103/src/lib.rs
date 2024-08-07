#![no_std]
#![no_main]

use core::marker::PhantomData;

use orbit::*;

// #[panic_handler]
// unsafe fn panic<'a, 'b>(_info: &'a PanicInfo<'b>) -> ! {
//     loop {
//         asm!("nop")
//     }
// }
