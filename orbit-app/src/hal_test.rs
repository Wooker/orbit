#![allow(static_mut_refs)]
#![allow(unsafe_code)]

pub struct HalTest {}

use crate::{application::Application, KERNEL};

#[used]
#[no_mangle]
#[link_section = ".apps"]
pub static HAL_TEST: HalTest = HalTest {};

pub struct HalTest;
impl Application<1> for HalTest {
    fn main(&self) {
        let mut gpio: Claimed<GPIO> = unsafe { KERNEL.claim().unwrap_unchecked() };
    }
}
