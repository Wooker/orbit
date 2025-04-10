use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};

use crate::app_stack;

app_stack!(32, "calc");

#[orbit_app()]
pub struct Calc {}

impl Calc {
    #[app_init("calc")]
    pub fn init(&mut self) {}
    #[app_interrupt("calc")]
    pub fn interrupt(&mut self) {}
    #[app_main("calc")]
    pub fn main(&mut self) -> usize {}
}
