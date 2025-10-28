#![no_std]
#![feature(trim_prefix_suffix)]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

use ch32x035_spi_driver::{Config, Spi as LibSpi};
use eink_lib_4_2::{Config as EinkConfig, Direction, Eink as LibEink, Position, Size};
use font_8x10::{self, ASCII};

app_heap!(0);
app_stack!(256);

const DC_PIN: u8 = 1;
const BUSY_PIN: u8 = 6;

const WIDTH: usize = 50;
const HEIGHT: usize = 30;
const LAYERS: usize = 10;

#[repr(C)]
struct Output([u8; 1]);

impl AsBytes for Output {
    type Output = Self;
    fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[orbit_app(GPIOA, SPI1)]
struct Reade {
    config: EinkConfig,
    letters: RingBuf<1875, RingbufType>,
    frame: RingBuf<15000, RingbufType>,
}
#[orbit_impl]
impl Reade {
    #[app_init("reade")]
    fn init(&mut self) {
        // let mut spi1 = unsafe { self.peripherals.spi1.assume_init_read() };
        // let mut gpioa = unsafe { self.peripherals.gpioa.assume_init_read() };

        self.letters.flush();
        self.frame.buf.iter_mut().for_each(|b| *b = 0);

        // let bus = LibSpi::new(&mut spi1, Config::default());
        // let mut eink: LibEink<DC_PIN, BUSY_PIN, 15000> = LibEink::new(bus, &mut gpioa);
        self.config = EinkConfig {
            pos: Position { x: 0, y: 0 },
            dir: Direction::XuYiXi,
            size: Size {
                width: 400,
                height: 300,
            },
            partial: false,
        };
        // eink.display(&self.config, &[0; 15000]);
    }

    #[app_interrupt("reade")]
    fn interrupt(&mut self) {}

    #[app_main("reade")]
    fn main(&mut self) -> Output {
        let mut spi1 = unsafe { self.peripherals.spi1.assume_init_read() };
        let mut gpioa = unsafe { self.peripherals.gpioa.assume_init_read() };

        let buf = unsafe { self.ringbuf.read().unwrap_unchecked() };
        if let Ok(arg) = str::from_utf8(buf) {
            let arg = &arg[..WIDTH];
            let len = arg.len();
            let cmp = if len >= 4 {
                arg[..4].cmp("show")
            } else {
                core::cmp::Ordering::Less
            };

            match cmp {
                core::cmp::Ordering::Equal => {
                    for line in 0..HEIGHT {
                        for layer in 0..LAYERS {
                            for ch in 0..WIDTH {
                                if let Some(pos) = self.letters.at(ch + (line * WIDTH)) {
                                    let pos = *pos as usize;
                                    let l = if let Some(ascii_ch) = ASCII.get(pos) {
                                        ascii_ch[layer]
                                    } else {
                                        ASCII[32][layer]
                                    };
                                    self.frame.buf
                                        [((layer * WIDTH) + ch) + (line * WIDTH * LAYERS)] = l;
                                }
                            }
                        }
                    }
                    let bus = LibSpi::new(&mut spi1, Config::default());
                    let mut eink: LibEink<DC_PIN, BUSY_PIN, 15000> = LibEink::new(bus, &mut gpioa);
                    eink.display(&self.config, &self.frame.buf);
                    self.letters.flush();
                    self.frame.flush();
                    Output([0])
                }
                _ => {
                    arg.as_bytes().iter().enumerate().for_each(|(_, b)| {
                        self.letters.push(*b);
                    });
                    Output([1])
                }
            }
        } else {
            Output([2])
        }
    }
}
