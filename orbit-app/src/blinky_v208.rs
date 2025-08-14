use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::{arch::delay, chip::pac::GPIOB, syscall::SysCall};

use crate::{app_stack, syscall};

app_stack!(64, "blinky");

#[orbit_app(GPIOB)]
pub struct Blinky {}

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[orbit_impl]
impl Blinky {
    #[app_init("blinky")]
    pub fn init(&mut self) {
        self.gpiob.modify(|p| {
            p.cfghr.write(|w| unsafe { w.bits(0b0101) });
            p.bshr.write(|w| unsafe { w.bits(1 << 8) });
        });
    }

    #[app_interrupt("blinky")]
    pub fn interrupt(&mut self) {}

    #[app_main("blinky")]
    pub fn main(&mut self) -> Output {
        let mut expr = [0u8; 1];
        // Parse the input from the application buffer
        {
            let arg = if let Some(msg) = self._buf.read() {
                unsafe { msg.split_last().unwrap_unchecked().1 }
            } else {
                &[1u8]
            };
            for (i, b) in arg.iter().enumerate() {
                expr[i] = *b;
            }
        }

        syscall!(SysCall::NumPorts);
        let _num_ports = if let Some(msg) = self._buf.read() {
            usize::from_le_bytes(unsafe {
                msg.split_last()
                    .unwrap_unchecked()
                    .1
                    .try_into()
                    .unwrap_unchecked()
            })
        } else {
            0
        };

        "\x01blinky ".bytes().for_each(|ch| self._buf.push(ch));
        self._buf.push(expr[0] + 5);
        self._buf.push(0);
        syscall!(SysCall::SendAll);

        self.gpiob.modify(|p| {
            p.bshr.write(|w| unsafe { w.bits(1 << 24) });
            delay(100000 * (expr[0] as u32));
            p.bshr.write(|w| unsafe { w.bits(1 << 8) });
        });

        Output([0])
    }
}
