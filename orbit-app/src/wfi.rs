#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{app_stack, app_struct, application::Application};
use core::arch::{asm, naked_asm};
use orbit_common_proc_macro::{app_init, app_interrupt, app_main};
use orbit_kernel::application::Context;

use core::sync::atomic::compiler_fence;

app_struct!(WFI: Wfi = Wfi::new(), "wfi");
app_stack!(4, "wfi");

pub struct Wfi {
    context: Context,
}

impl Wfi {
    pub const fn new() -> Self {
        Self {
            context: Context::new(),
        }
    }

    #[app_init("wfi")]
    pub fn init(&mut self) {}

    #[app_interrupt("wfi")]
    pub fn interrupt(&mut self) {}
}

#[app_main("wfi", Wfi)]
fn main(&mut self) {}
