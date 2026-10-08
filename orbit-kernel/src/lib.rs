#![cfg_attr(not(test), no_std)]
#![cfg_attr(not(test), no_main)]
#![feature(fn_align)]
#![feature(naked_functions_rustic_abi)]
#![feature(linked_list_cursors)]
#![feature(linked_list_remove)]
#![feature(ptr_metadata)]
#![feature(const_option_ops)]
#![feature(const_trait_impl)]
#![feature(const_result_trait_fn)]

//! Orbit kernel

pub mod application;
pub mod application_container;
pub mod context;
pub mod id;
pub mod ringbuf;
mod scheduler;
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
pub use orbit_arch as arch;
#[cfg(feature = "rt")]
pub use orbit_chip as chip;
#[cfg(feature = "rt")]
pub mod claim;
#[cfg(feature = "rt")]
pub mod containers;
pub mod task;

// TODO: make use of PMP internal. Application code should
// not rely on this const. to_container does rely at the moment
pub const PMP: usize = 4;

#[cfg(not(test))]
#[cfg(feature = "rt")]
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
