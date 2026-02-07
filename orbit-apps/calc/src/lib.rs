#![no_std]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};

app_heap!(32);
app_stack!(32);

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

#[orbit_app]
struct Calc;
#[orbit_impl]
impl Calc {
    #[app_init]
    pub fn init() {}

    #[app_interrupt]
    pub fn interrupt(&mut self) {}

    #[inline(never)]
    fn calc_expr(expr: [u8; 3]) -> u8 {
        let left = match expr[0] {
            b'1' => 1,
            b'2' => 2,
            b'3' => 3,
            b'4' => 4,
            b'5' => 5,
            b'6' => 6,
            b'7' => 7,
            b'8' => 8,
            b'9' => 9,
            _ => 0,
        };
        let right = match expr[2] {
            b'1' => 1,
            b'2' => 2,
            b'3' => 3,
            b'4' => 4,
            b'5' => 5,
            b'6' => 6,
            b'7' => 7,
            b'8' => 8,
            b'9' => 9,
            _ => 0,
        };
        match expr[1] {
            b'+' => left + right,
            b'-' => left - right,
            b'*' => left * right,
            b'/' => left / right,
            _ => 0,
        }
    }
    #[app_main]
    pub fn main(&mut self) -> Output {
        let mut argument = [0u8; 3];
        for (i, b) in unsafe { self.ringbuf.read().iter().enumerate().take(3) } {
            argument[i] = *b;
        }
        Output([Self::calc_expr(argument)])
    }
}
