// While compiling with  rustc 1.91.0-nightly (54c581243 2025-08-25)
// cargo produces:
// ```
// warning: `#[link_section]` attribute cannot be used on inherent methods
// ```
// If such behavior is no longer observable on newer versions of rustc,
// remove this attribute
#![allow(unused_attributes)]

use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::syscall::SysCall;

use crate::{app_stack, syscall};

app_stack!(32, "calc");

#[orbit_app()]
pub struct Calc {}

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
    #[app_init("calc")]
    pub fn init(&mut self) {}

    #[app_interrupt("calc")]
    pub fn interrupt(&mut self) {}

    #[inline(never)]
    #[link_section = ".calc.text"]
    fn calc_expr(expr: [u8; 3]) -> u8 {
        match expr[1] {
            b'+' => expr[0] + expr[2],
            b'-' => expr[0] - expr[2],
            b'*' => expr[0] * expr[2],
            b'/' => expr[0] / expr[2],
            _ => 0,
        }
    }

    #[app_main("calc")]
    pub fn main(&mut self) -> Output {
        // Currently expressions in the following forms are supported:
        // "expr operand expr"
        // where expr is "number_operand_number"

        // Parse the input from the application buffer
        let mut argument = [0u8; 9];
        {
            let arg = if let Some(msg) = self._buf.read() {
                unsafe { msg.split_last().unwrap_unchecked().1 }
            } else {
                &[0u8]
            };
            for (i, b) in arg.iter().enumerate() {
                argument[i] = *b;
            }
        }

        self._buf.flush();
        syscall!(SysCall::NumPorts);
        let num_ports = usize::from_le_bytes(unsafe {
            self._buf
                .read()
                .unwrap_unchecked()
                .split_last()
                .unwrap_unchecked()
                .1
                .try_into()
                .unwrap_unchecked()
        });

        if num_ports == 1 {
            if let Some((3, _)) = argument.iter().enumerate().find(|(_, b)| **b == 0) {
                let expr1 = [argument[0], argument[1], argument[2]];
                Output([Self::calc_expr(expr1)])
            } else {
                let expr1 = [argument[0], argument[1], argument[2]];
                let expr2 = [argument[6], argument[7], argument[8]];

                let val1 = Self::calc_expr(expr1);
                let val2 = Self::calc_expr(expr2);

                let expr3 = [val1, argument[4], val2];
                Output([Self::calc_expr(expr3)])
            }
        } else {
            if let Some((3, _)) = argument.iter().enumerate().find(|(_, b)| **b == 0) {
                let expr1 = [argument[0], argument[1], argument[2]];
                Output([Self::calc_expr(expr1)])
            } else {
                let mut expr1 = [0u8; 3];
                let mut operator = [0u8; 1];
                let mut expr2 = [0u8; 3];

                for (i, b) in argument[..3].iter().enumerate() {
                    expr1[i] = *b;
                }
                for (i, b) in argument[6..].iter().enumerate() {
                    expr2[i] = *b;
                }
                operator[0] = argument[4];

                let mut call = [0u8; 10];
                for (i, b) in b"\x01calc ".iter().enumerate() {
                    call[i] = *b;
                }
                call[6] = expr2[0];
                call[7] = expr2[1];
                call[8] = expr2[2];

                call.iter().for_each(|ch| self._buf.push(*ch));
                syscall!(SysCall::SendAll);

                // Calculate the expr1
                let val1 = Self::calc_expr(expr1);

                syscall!(SysCall::Await);
                let val2 = unsafe {
                    self._buf
                        .read()
                        .unwrap_unchecked()
                        .split_last()
                        .unwrap_unchecked()
                        .1[0]
                };

                expr1[0] = val1;
                expr1[1] = operator[0];
                expr1[2] = val2;

                Output([Self::calc_expr(expr1)])
            }
        }
    }
}
