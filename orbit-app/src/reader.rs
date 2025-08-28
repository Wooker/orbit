use orbit_common::{app_heap, app_stack};
use orbit_common_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

#[allow(unused)]
use orbit_libos::{
    eink::{Config as EinkConfig, Direction, Eink as LibEink, Position, Size},
    font_8x10::{self, Letter, ASCII},
    spi::{Config, Spi as LibSpi},
};

app_heap!(0);
app_stack!(32);

const DC_PIN: u8 = 1;
const BUSY_PIN: u8 = 6;

const WIDTH: usize = 25;
const HEIGHT: usize = 20;
const LAYERS: usize = 10;

#[orbit_app(GPIOA, SPI1)]
struct Reade {
    letters: RingBuf<625, u8>,
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
impl Reade {
    #[app_init("reade")]
    fn init(&mut self) {
        let bus = LibSpi::new(&mut self.spi1, Config::default());
        let mut eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut self.gpioa);
        let config = EinkConfig {
            pos: Position { x: 0, y: 0 },
            dir: Direction::XuYiXi,
            size: Size {
                width: 200,
                height: 200,
            },
        };
        eink.display(config, &self.frame.buf, false);
    }

    #[app_interrupt("reade")]
    fn interrupt(&mut self) {}

    #[app_main("reade")]
    fn main(&mut self) -> Output {
        let bus = LibSpi::new(&mut self.spi1, Config::default());
        let mut eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut self.gpioa);
        if self._buf.buf[..4].cmp(b"show") != core::cmp::Ordering::Equal {
            let letter = self._buf.buf[0];
            self._buf.buf[..25]
                .iter()
                .for_each(|b| self.letters.push(*b));
            self._buf.flush();
            Output { 0: [letter] }
        } else {
            for line in 0..HEIGHT {
                for layer in 0..LAYERS {
                    for ch in 0..WIDTH {
                        let pos = self.letters.buf[ch + (line * WIDTH)] as usize;
                        self.frame.buf[((layer * WIDTH) + ch) + (line * WIDTH * LAYERS)] =
                            ASCII[pos][layer];
                    }
                }
            }
            let config = EinkConfig {
                pos: Position { x: 0, y: 0 },
                dir: Direction::XuYiXi,
                size: Size {
                    width: 200,
                    height: 200,
                },
            };
            eink.display(config, &self.frame.buf, false);
            self.letters.flush();
            self.frame.flush();
            self._buf.flush();
            Output { 0: [1] }
        }
    }
}
