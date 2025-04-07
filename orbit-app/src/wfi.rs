#![allow(static_mut_refs)]
#![allow(unsafe_code)]

use crate::{app_stack, application::Application};
use core::arch::{asm, naked_asm};
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};

use core::sync::atomic::compiler_fence;

app_stack!(4, "wfi");

#[orbit_app()]
pub struct Wfi {}

impl Wfi {
    #[app_init("wfi")]
    pub fn init(&mut self) {}

    #[app_interrupt("wfi")]
    pub fn interrupt(&mut self) {}
}

#[app_main("wfi", Wfi)]
fn main(&mut self) {}
