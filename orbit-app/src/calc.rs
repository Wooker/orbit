use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};
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

impl Calc {
    #[app_init("calc")]
    pub fn init(&mut self) {}

    #[app_interrupt("calc")]
    pub fn interrupt(&mut self) {}

    #[app_main("calc")]
    pub fn main(&mut self) -> Output {
        // Currently expressions in the following forms are supported:
        // hex+hex
        // hex-hex
        // hex*hex
        // hex/hex

        let mut expr = [0u8; 3];
        // Parse the input from the application buffer
        {
            let arg = if let Some(msg) = self._buf.read() {
                unsafe { msg.split_last().unwrap_unchecked().1 }
            } else {
                &[0u8]
            };
            for (i, b) in arg.iter().enumerate() {
                expr[i] = *b;
            }
        }

        let msg = b"\x01blinky \x25";
        for ch in msg {
            self._buf.push(*ch);
        }
        self._buf.push(self._buf.termination);

        syscall!(SysCall::SendAll);

        // Calculate the output
        match expr[1] {
            b'+' => Output([expr[0] + expr[2]]),
            b'-' => Output([expr[0] - expr[2]]),
            b'*' => Output([expr[0] * expr[2]]),
            b'/' => Output([expr[0] / expr[2]]),
            // _ => Output([u8::MAX]),
            _ => Output([0 as u8]),
        }
    }
}
