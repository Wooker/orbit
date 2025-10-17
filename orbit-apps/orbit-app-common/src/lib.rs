#![no_std]

pub trait AsBytes {
    type Output;
    fn as_bytes(&self) -> &[u8];
}

impl AsBytes for () {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        unsafe {
            core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            )
        }
    }
}

#[macro_export]
macro_rules! app_stack {
    ($size:expr) => {
        const STACK_SIZE: usize = $size;
        const_assert!(STACK_SIZE >= 1);
    };
}
#[macro_export]
macro_rules! app_heap {
    ($size:expr) => {
        const HEAP_SIZE: usize = $size;
        const_assert!(HEAP_SIZE >= 0);
    };
}

#[macro_export]
macro_rules! syscall {
    ($syscall:path) => {
        unsafe {
            asm!(
                // Store argument registers' values on stack
                "addi sp, sp, -0x10;",
                "sw a0, 0x0(sp);",
                "sw a1, 0x4(sp);",
                "sw a2, 0x8(sp);",
                "sw a3, 0xc(sp);",

                // Send the syscall via ECALL
                "li a1, {syscall}",
                "li a0, 0",
                "ecall",

                // Restore argument registers' values from stack
                "lw a0, 0x0(sp);",
                "lw a1, 0x4(sp);",
                "lw a2, 0x8(sp);",
                "lw a3, 0xc(sp);",
                "addi sp, sp, 0x10;",
                syscall = const ($syscall.discriminant()),
            );
        }
    };
}
