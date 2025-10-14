#![no_std]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};

app_heap!(32);
app_stack!(32);

#[orbit_app]
struct Test-app {}

#[orbit_impl]
impl Test-app {
    #[app_init]
    pub fn init(&mut self) {}

    #[app_interrupt]
    pub fn interrupt(&mut self) {}

    #[app_main]
    pub fn main(&mut self) {}
}