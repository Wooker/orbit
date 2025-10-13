#![no_std]

use orbit_app_common::{app_heap, app_stack, syscall};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::syscall::SysCall;

app_heap!(32);
app_stack!(32);

#[orbit_app]
struct Calc {}

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[orbit_impl]
impl Calc {
    #[app_init]
    pub fn init(&mut self) {}

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
    pub fn _main() -> Output {
        Output([0])
    }
}
// #[app_main]
// pub fn main(&mut self) -> Output {
//     let mut argument = [0u8; 3];
//     {
//         let arg = if let Some(msg) = self._buf.read() {
//             msg
//         } else {
//             &[0u8]
//         };
//         for (i, b) in arg.iter().enumerate().take(3) {
//             argument[i] = *b;
//         }
//     }
//     self._buf.flush();
//     Output([Self::calc_expr(argument)])
// }
