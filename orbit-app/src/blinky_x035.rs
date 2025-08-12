use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};
use orbit_kernel::chip::pac::GPIOB;

use crate::app_stack;

use orbit_kernel::arch;

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

impl Blinky {
    #[app_init("blinky")]
    pub fn init(&mut self) {
        self.gpiob.modify(|p| {
            p.cfghr().write(|w| unsafe { w.bits(0b0001 << 16) });
        });
    }

    #[app_interrupt("blinky")]
    pub fn interrupt(&mut self) {}

    #[app_main("blinky")]
    pub fn main(&mut self) -> Output {
        let arg = if let Some(msg) = self._buf.read() {
            unsafe { msg.split_last().unwrap_unchecked().1 }
        } else {
            &[0u8]
        };

        self.gpiob.modify(|p| {
            p.bshr().write(|w| unsafe { w.bits(1 << 12) });
            arch::delay(100000 * (arg[0] as u32));
            p.bshr().write(|w| unsafe { w.bits(1 << 28) });
        });

        Output([0])
    }
}
