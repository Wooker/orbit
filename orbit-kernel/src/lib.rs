#![no_std]
#![no_main]

pub mod kernel;

pub use orbit_arch::arch;

#[panic_handler]
pub fn panic_handler<'a, 'b>(_: &'a core::panic::PanicInfo<'b>) -> ! {
    loop {
        // arch::qingke::riscv::asm::wfi();
    }
}

#[no_mangle]
pub fn ExceptionHandler() {}
