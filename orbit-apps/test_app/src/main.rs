#![no_std]
#![no_main]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};

app_heap!(32);
app_stack!(32);

#[orbit_app]
pub struct TestApp {}

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[orbit_impl]
impl TestApp {
    #[app_init]
    pub fn init(&mut self) {}

    #[app_interrupt]
    pub fn interrupt(&mut self) {}

    #[app_main]
    pub fn main(&mut self) -> Output {
        let a = test_libos::test_libos_add(1, 2);
        Output([a as u8])
    }
}
