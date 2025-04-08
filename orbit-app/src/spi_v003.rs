#![allow(unsafe_code)]
use crate::{app_stack, application::Application};
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app};
use orbit_kernel::chip::pac::SPI1;

app_stack!(128, "spi");

#[orbit_app(SPI1)]
pub struct Spi {}

impl Spi {
    #[app_init("spi")]
    fn init(&mut self) {
        let spi = unsafe { self.spi1.assume_init_mut() };

        spi.modify(|p| {
            let ctlr1 = 1 << 14 | 1 << 6 | 1 << 2;

            p.ctlr1.write(|w| unsafe { w.bits(ctlr1) });
        });
    }
}
#[app_main("spi", Spi)]
fn main(&mut self) {
    let spi = unsafe { self.spi1.assume_init_mut() };
}
