#![no_std]
#![no_main]
#![allow(static_mut_refs)]

pub mod application;
pub mod service;

use orbit_common::feature_mod;
use orbit_kernel::kernel::Kernel;

#[allow(unsafe_code)]
unsafe extern "Rust" {
    pub static mut KERNEL: Kernel<'static>;
}

pub mod calc;
pub mod system_num_ports;
pub mod wfi;

feature_mod!("ch592", pub, blinky);
feature_mod!("ch32v208wbu6", pub, blinky_v208);
feature_mod!("ch32v003", pub, blinky_v003);
feature_mod!("ch32x035", pub, blinky_x035);

/// Macros

#[macro_export]
macro_rules! app_stack {
    ($size:expr, $app_name:expr) => {
        #[allow(unused)]
        #[link_section = concat!(".", $app_name, ".bss")]
        pub static mut STACK: [usize; $size] = [0; $size];
    };
}

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
