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

impl AsBytes for u8 {
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
                "li a0, {syscall}",
                "li a1, 0",
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

#[macro_export]
macro_rules! send {
    ($buf:expr) => {
        unsafe {
            // By binding to a local variable 'data' here, we force the
            // temporary to live until the end of this unsafe block.
            let data = $buf;
            let ptr = data.as_ptr();
            let len = data.len();
            let syscall = SysCall::Send.discriminant();

            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") ptr,
                in("a2") len,
                clobber_abi("C"),
            );
        }
    };
}

#[macro_export]
macro_rules! register_interrupt {
    ($int:expr) => {
        unsafe {
            // By binding to a local variable 'data' here, we force the
            // temporary to live until the end of this unsafe block.
            let interrupt = $int;
            let syscall = SysCall::RegisterInterrupt.discriminant();

            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") interrupt,
                clobber_abi("C"),
            );
        }
    };
}

#[macro_export]
macro_rules! invoke {
    ($app:expr, $buf:expr) => {{
        let result: usize;
        unsafe {
            let app = $app;
            let aptr = app.as_ptr();
            let alen = app.len();
            let data = $buf;
            let ptr = data.as_ptr();
            let len = data.len();
            let syscall = SysCall::Invoke.discriminant();

            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") aptr,
                in("a2") alen,
                in("a3") ptr,
                in("a4") len,
                lateout("a0") result,
                clobber_abi("C"),
            );
        }
        result
    }};
}
