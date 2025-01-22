use core::arch::global_asm;

#[cfg(feature = "qingke_v4")]
pub use qingke::*;
#[cfg(feature = "qingke_v4")]
pub use qingke_rt::entry;

#[cfg(feature = "riscv")]
pub use riscv_rt_macros::entry;

#[cfg(feature = "riscv")]
pub use riscv;
// #[cfg(feature = "riscv")]
// pub use riscv_rt::entry;

#[cfg(feature = "riscv")]
global_asm!(
    "
.section .init;
.global _start;

_start:
    csrc mstatus, 0x0

    // Check hart ID
    // csrr t2, mhartid
    // lui t0, %hi(_max_hart_id)
    // add t0, t0, %lo(_max_hart_id)
    // bgtu t2, t0, abort

    // Allocate stack
    la sp, _stack_start
    li t0, 4 // make sure stack start is in RAM
    sub sp, sp, t0
    andi sp, sp, -16 // Force 16-byte alignment

    // Set frame pointer
    add s0, sp, zero

    la t0, main
    csrw mepc, t0

    mret

abort:
    j abort;
"
);

#[no_mangle]
fn _start_trap() {}
