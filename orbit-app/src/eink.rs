use orbit_common::{app_heap, app_stack};
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

#[allow(unused)]
use orbit_libos::{
    eink::{Config as EinkConfig, Direction, Eink as LibEink, Position, Size},
    font_8x8::{self, Letter},
    spi::{Config, Spi as LibSpi},
};

app_heap!(0);
app_stack!(64);

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
            let config = EinkConfig {
                pos: Position { x: 0, y: 0 },
                dir: Direction::XuYiXi,
                size: Size {
                    width: 200,
                    height: 200,
                },
            };

            eink.display(config, &self.frame.buf, false);
            self.frame.flush();
            self._buf.flush();
            Output { 0: [1] }
        }
    }
}
