#![no_std]
#![no_main]
#![feature(fn_align)]
#![feature(naked_functions_rustic_abi)]
#![feature(linked_list_cursors)]
#![feature(ptr_metadata)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]

//! Orbit kernel

pub mod application;
pub mod application_container;
pub mod claim;
pub mod clock;
pub mod context;
pub mod id;
pub mod ringbuf;
pub mod syscall;
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
pub mod task;
#[cfg(feature = "rt")]
pub use chip;
#[cfg(feature = "rt")]
pub use orbit_arch as arch;
use spaceport::packet::MAX_BUFFER_LENGTH;

// TODO: make use of PMP internal. Application code should
// not rely on this const. to_container does rely at the moment
pub const PMP: usize = 4;
pub const RINGBUF_SIZE: usize = MAX_BUFFER_LENGTH;

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
