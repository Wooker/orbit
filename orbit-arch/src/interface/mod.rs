//! Core modules
//!

pub mod cpu;
pub use cpu::*;
pub mod debug;
pub mod ic;
pub mod pmp;
pub mod timer;

pub trait Interruptable {
    fn enable_interrupt(&self, interrupt_code: usize);
    fn disable_interrupt(&self, interrupt_code: usize);
}

pub trait SetupMtvec {
    type TrapMode;
    fn setup_mtvec(&self, addr: usize, mode: Self::TrapMode);
}
