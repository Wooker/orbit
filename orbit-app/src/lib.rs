#![no_std]
#![no_main]

pub mod application;

use orbit_common::feature_mod;

pub mod calc;
pub mod system_num_ports;
pub mod wfi;

feature_mod!("ch592", pub, blinky);
feature_mod!("ch32v208wbu6", pub, blinky_v208);
feature_mod!("ch32v003", pub, blinky_v003);
feature_mod!("ch32x035", pub, blinky_x035);
feature_mod!("ch32x035", pub, eink);
feature_mod!("ch32x035", pub, reader);

/// Macros

#[macro_export]
macro_rules! syscall {
    ($syscall:path) => {
        unsafe {
            asm!(
                "
                addi sp, sp, -0x10;
                sw a0, 0x0(sp);
                sw a1, 0x4(sp);
                sw a2, 0x8(sp);
                sw a3, 0xc(sp);
                ",
                "li a1, {syscall}",
                "li a0, 0",
                "ecall",
                "
                lw a0, 0x0(sp);
                lw a1, 0x4(sp);
                lw a2, 0x8(sp);
                lw a3, 0xc(sp);
                addi sp, sp, 0x10;
                ",
                syscall = const ($syscall.discriminant()),
            );
        }
    };
}
