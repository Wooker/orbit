use crate::app_stack;
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};

app_stack!(4, "wfi");

#[orbit_app()]
pub struct Wfi {}

#[orbit_impl]
impl Wfi {
    #[app_init("wfi")]
    pub fn init(&mut self) {}

    #[app_interrupt("wfi")]
    pub fn interrupt(&mut self) {}

    #[app_main("wfi")]
    pub fn main(&mut self) {}
}
