static mut IRQ_STA: usize = 0;

use core::ptr;

use orbit_arch::register::gintenr;

pub fn with_safe_access<F, R>(f: F) -> R
where
    F: FnOnce() -> R,
{
    const REG_SAFE_ACCESS_SIG: *mut u8 = 0x40001040 as *mut u8;
    const SAFE_ACCESS_SIG1: u8 = 0x57;
    const SAFE_ACCESS_SIG2: u8 = 0xA8;

    unsafe {
        if gintenr::read() & 0x08 != 0 {
            IRQ_STA = gintenr::read();
            gintenr::write(IRQ_STA & (!0x08));
        }
        orbit_arch::riscv::asm::nop();
        orbit_arch::riscv::asm::nop();

        ptr::write_volatile(REG_SAFE_ACCESS_SIG, SAFE_ACCESS_SIG1);
        ptr::write_volatile(REG_SAFE_ACCESS_SIG, SAFE_ACCESS_SIG2);

        orbit_arch::riscv::asm::nop();
        orbit_arch::riscv::asm::nop();
    }
    let ret = f();
    unsafe {
        ptr::write_volatile(REG_SAFE_ACCESS_SIG, 0);
        gintenr::write(gintenr::read() | (IRQ_STA & 0x08));
        IRQ_STA = 0;
        orbit_arch::riscv::asm::nop();
        orbit_arch::riscv::asm::nop();
    }
    ret
}
