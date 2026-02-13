#![no_std]
#![no_main]
#![feature(stmt_expr_attributes)]
#![feature(ascii_char)]
#![feature(concat_bytes)]
#![feature(const_index)]
#![feature(const_trait_impl)]
#![feature(fn_align)]
#![feature(naked_functions_rustic_abi)]
#![feature(slice_split_once)]
#![feature(iter_array_chunks)]
#![feature(slice_shift)]

pub mod application;
pub mod application_container;
pub mod claim;
pub mod clock;
pub mod context;
pub mod ringbuf;
pub mod syscall;
pub mod task;
pub mod usizebuf;

pub use spaceport;

extern crate alloc;
#[cfg(feature = "rt")]
mod allocator;
#[cfg(feature = "rt")]
pub mod kernel;
#[cfg(feature = "rt")]
pub mod pmp_entry;
#[cfg(feature = "rt")]
pub mod port;
#[cfg(feature = "rt")]
pub use chip;
#[cfg(feature = "rt")]
pub use orbit_arch as arch;

// TODO: make use of PMP internal. Application code should
// not rely on this const. to_container does rely at the moment
pub const PMP: usize = 4;
pub const RINGBUF_SIZE: usize = 64;
pub type RingbufType = u8;

// #[cfg(feature = "rt")]
#[panic_handler]
pub fn panic_handler<'a, 'b>(_info: &'a core::panic::PanicInfo<'b>) -> ! {
    // unsafe { save_context() };
    // use crate::kernel::{Kernel, asm::save_context};

    // let mut kernel_addr: usize;
    // unsafe {
    //     core::arch::asm!("csrr {0}, mscratch", out(reg) kernel_addr);
    // }
    // let kernel = unsafe { &mut *(kernel_addr as *mut Kernel) };
    // kernel.handle_panic(info);
    loop {}
}

#[cfg(feature = "rt")]
use core::arch::global_asm;
#[cfg(feature = "rt")]
global_asm!(
    "
    .section .init
    .global _start

_start:
    la sp, _stack_top
    ",
    // "
    // li t0, 0x1f
    // csrw 0xbc0, t0
    // li t0, 0x08
    // csrw 0x804, t0
    // ",
    "li t0, 0x1880",
    "csrw mstatus, t0",
    "la ra, initialize_finish",
    // Set mtvec
    "
    la t0, handler;
    csrw mtvec, t0;
    ",
    // Set dcsr 9 and 11 bits
    "
    csrr t0, dcsr;
    li t1, 0xa00;
    or t0, t0, t1;
    csrw dcsr, t0;
    ",
    "
    la t0, main;
    csrw mepc, t0;
    mret;
    // j main;
    "
);
