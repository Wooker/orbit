use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

#[allow(unused)]
use orbit_libos::{
    eink::Eink as LibEink,
    font_8x8::{self, Letter},
    spi::{Config, Spi as LibSpi},
};

use crate::app_stack;

app_stack!(64, "eink");
const DC_PIN: u8 = 1;
const BUSY_PIN: u8 = 6;

#[orbit_app(GPIOA, SPI1)]
struct Eink {
    frame: RingBuf<5000, u8>,
}

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[orbit_impl]
impl Eink {
    #[app_init("eink")]
    fn init(&mut self) {}

    #[app_interrupt("eink")]
    fn interrupt(&mut self) {}

    #[app_main("eink")]
    fn main(&mut self) -> Output {
        let bus = LibSpi::new(&mut self.spi1, Config::default());
        let mut eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut self.gpioa);

        if self._buf.buf[..4].cmp(b"show") != core::cmp::Ordering::Equal {
            self._buf.buf[..26].iter().for_each(|b| self.frame.push(*b));
            self._buf.flush();
            Output { 0: [2] }
        } else {
            eink.display(&self.frame.buf, false);
            self.frame.flush();
            self._buf.flush();
            Output { 0: [1] }
        }
    }
}
