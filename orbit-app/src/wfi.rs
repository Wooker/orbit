#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{app_stack, app_struct, application::Application};
use core::arch::asm;
use orbit_kernel::application::Context;

app_struct!(WFI: Wfi = Wfi::new(), "wfi");
app_stack!(4, "wfi");

pub struct Wfi(Context);
impl Wfi {
    pub const fn new() -> Self {
        Self(Context::new())
    }
    pub fn init(&mut self) {}
}
impl Application for Wfi {
    fn main(&mut self) {
        unsafe { asm!("wfi") };
    }

    #[inline(never)]
    #[link_section = ".wfi.text"]
    fn context(&self) -> Context {
        self.0
    }
}
