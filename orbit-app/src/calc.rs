use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};
use orbit_kernel::syscall::SysCall;

use crate::app_stack;

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

        unsafe {
            asm!(
                "
            addi sp, sp, -0x8;
            sw a0, 0x0(sp);
            sw a1, 0x4(sp);
            "
            );
        }
        let syscall: usize = SysCall::NumPorts.into();
        unsafe {
            asm!(
                "
            li a0, 0;
            
            ",
                in("a1") syscall
            );
            asm!("ecall");
        }

        unsafe {
            asm!(
                "
            sw a0, 0x0(sp);
            sw a1, 0x4(sp);
            addi sp, sp, 0x8;
            "
            )
        }

        let mut num_ports = {
            let bytes = if let Some(msg) = self._buf.read() {
                unsafe {
                    msg.split_last()
                        .unwrap_unchecked()
                        .1
                        .first_chunk::<4>()
                        .unwrap_unchecked()
                }
            } else {
                &[0u8; 4]
            };
            usize::from_le_bytes(*bytes)
        };

        // Calculate the output
        match expr[1] {
            b'+' => Output([expr[0] + expr[2]]),
            b'-' => Output([expr[0] - expr[2]]),
            b'*' => Output([expr[0] * expr[2]]),
            b'/' => Output([expr[0] / expr[2]]),
            // _ => Output([u8::MAX]),
            _ => Output([num_ports as u8]),
        }
    }
}
