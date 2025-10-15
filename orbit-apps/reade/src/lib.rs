#![no_std]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

use ch32x035_spi_driver::{Config, Spi as LibSpi};
use eink_lib::{Config as EinkConfig, Direction, Eink as LibEink, Position, Size};
use font_8x10::{self, ASCII};

app_heap!(0);
app_stack!(128);

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
    fn init(&mut self) {}

    #[app_interrupt("reade")]
    fn interrupt(&mut self) {}

    #[app_main("reade")]
    fn main(&mut self) -> Output {
        if let Some(arg) = self._buf.read() {
            let arg = str::from_utf8(&arg).unwrap().trim();
            let cmp = arg[..4].cmp("show");

            let mut gpioa = Self::claim_peripheral_gpioa();
            let mut gpioa = Claimed::new(&mut gpioa);
            let mut spi1 = Self::claim_peripheral_spi1();
            let mut spi1 = Claimed::new(&mut spi1);

            let bus = LibSpi::new(&mut spi1, Config::default());
            let mut eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut gpioa);

            match cmp {
                core::cmp::Ordering::Equal => {
                    self.letters.buf[0] = b'a';
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
<<<<<<< HEAD
                        pos: Position {
                            x: math::add_t::<usize>(0, 0),
                            y: 0,
                        },
=======
                        pos: Position { x: 0, y: 0 },
>>>>>>> 73cce2c (wip: errors in kernel-app communications)
                        dir: Direction::XuYiXi,
                        size: Size {
                            width: 200,
                            height: 200,
                        },
                    };
<<<<<<< HEAD
=======
                    let mut gpioa = Self::claim_peripheral_gpioa();
                    let mut gpioa = Claimed::new(&mut gpioa);
                    let mut spi1 = Self::claim_peripheral_spi1();
                    let mut spi1 = Claimed::new(&mut spi1);
                    let bus = LibSpi::new(&mut spi1, Config::default());
                    let mut eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut gpioa);
>>>>>>> 73cce2c (wip: errors in kernel-app communications)
                    eink.display(config, &self.frame.buf, false);
                    self.letters.flush();
                    self.frame.flush();
                    self._buf.flush();
                    Output([1])
                }
                _ => {
                    let chars = arg.chars();
<<<<<<< HEAD
                    chars.clone().for_each(|b| self.letters.push(b as u8));
                    // self._buf.buf[..25]
                    //     .iter()
                    //     .for_each(|b| self.letters.push(*b));
                    // self._buf.flush();
                    Output([chars.count() as u8])
=======
                    chars.for_each(|b| self.letters.push(b as u8));

                    // self._buf.buf[..25]
                    //     .iter()
                    //     .for_each(|b| self.letters.push(*b));
                    self._buf.flush();
                    Output([2])
>>>>>>> 73cce2c (wip: errors in kernel-app communications)
                }
            }
        } else {
            Output([0xff])
        }
    }
}
