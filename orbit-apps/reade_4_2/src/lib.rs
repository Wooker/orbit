#![no_std]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

use ch32x035_spi_driver::{Config, Spi as LibSpi};
use eink_lib_4_2::{Config as EinkConfig, Direction, Eink as LibEink, Position, Size};
use font_8x8::{self, ASCII, ASCII_ROTATED};

app_heap!(0);
app_stack!(256);

const DC_PIN: u8 = 1;
const BUSY_PIN: u8 = 6;

const WIDTH: usize = 37;
const HEIGHT: usize = 50;
const LAYERS: usize = 8;

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
        let mut spi1 = unsafe { self.peripherals.spi1.assume_init_read() };
        let mut gpioa = unsafe { self.peripherals.gpioa.assume_init_read() };

        self.letters.flush();
        self.frame.buf.iter_mut().for_each(|b| *b = 0);

        let bus = LibSpi::new(&mut spi1, Config::default());
        let mut eink: LibEink<DC_PIN, BUSY_PIN, 15000> = LibEink::new(bus, &mut gpioa);
        self.config = EinkConfig {
            pos: Position { x: 0, y: 0 },
            dir: Direction::XuYiXi,
            size: Size {
                width: 400,
                height: 300,
            },
            partial: false,
        };
        eink.display(&self.config, &self.frame.buf);
        self.config = EinkConfig {
            pos: Position { x: 399, y: 0 },
            dir: Direction::YuYiXd,
            size: Size {
                width: 400,
                height: 300 - (300 % 8),
            },
            partial: false,
        };
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
            let cmp = if len >= 5 {
                arg[..5].cmp("\\show")
            } else {
                core::cmp::Ordering::Less
            };

            match cmp {
                core::cmp::Ordering::Equal => {
                    // Landscape
                    for line in 0..HEIGHT {
                        for ch in 0..WIDTH {
                            for layer in 0..LAYERS {
                                if let Some(pos) = self.letters.at(ch + (line * WIDTH)) {
                                    let pos = *pos as usize;
                                    let l = if let Some(ascii_ch) = ASCII_ROTATED.get(pos) {
                                        // Rotated::new(*ascii_ch).0[layer]
                                        ascii_ch[layer]
                                    } else {
                                        ASCII_ROTATED[32][layer]
                                    };
                                    self.frame.buf
                                        [(layer + (ch * LAYERS)) + (line * WIDTH * LAYERS)] = l;
                                }
                            }
                        }
                    }
                    // Landscape 180
                    // for line in 0..HEIGHT {
                    //     for layer in 0..LAYERS {
                    //         for ch in 0..WIDTH {
                    //             if let Some(pos) = self.letters.at(ch + (line * WIDTH)) {
                    //                 let pos = *pos as usize;
                    //                 let mut l = if let Some(ascii_ch) = ASCII.get(pos) {
                    //                     ascii_ch[layer]
                    //                 } else {
                    //                     ASCII[32][layer]
                    //                 };
                    //                 let mut new_l = 0;
                    //                 for _ in 0..7 {
                    //                     new_l |= (l & 0b1);
                    //                     l >>= 1;
                    //                     new_l <<= 1;
                    //                 }
                    //                 new_l |= (l & 0b1);
                    //                 l = new_l;
                    //                 self.frame.buf[self.frame.buf.len()
                    //                     - 1
                    //                     - (((layer * WIDTH) + ch) + (line * WIDTH * LAYERS))] = l;
                    //             }
                    //         }
                    //     }
                    // }
                    //
                    // Portrait rotate right
                    // for line in (0..HEIGHT).rev() {
                    //     for ch in 0..WIDTH {
                    //         if let Some(p) = self.letters.at(ch + (line * WIDTH)) {
                    //             let pos = *p as usize;
                    //             let mut l = if let Some(ascii_ch) = ASCII.get(pos) {
                    //                 let mut nl = 0;
                    //                 for n in 9..1 {
                    //                     nl |= ascii_ch[n];
                    //                     nl <<= 1;
                    //                 }
                    //                 nl
                    //             } else {
                    //                 0
                    //             };
                    //             self.frame.buf[0] = l;
                    //         }
                    //     }
                    // }

                    // for i in 0..256 {
                    //     self.frame.buf[i] = i as u8;
                    // }
                    let bus = LibSpi::new(&mut spi1, Config::default());
                    let mut eink: LibEink<DC_PIN, BUSY_PIN, 15000> = LibEink::new(bus, &mut gpioa);
                    eink.display(&self.config, &self.frame.buf);
                    self.letters.flush();
                    self.frame.flush_with(0);
                    Output([0])
                }
                _ => {
                    arg.as_bytes().iter().enumerate().for_each(|(_, b)| {
                        self.letters.push(*b);
                        // self.frame
                        //     .push_at(self.config.pos.x + (self.config.pos.y * WIDTH) + i, *b);
                    });
                    Output([1])
                }
            }
        } else {
            Output([2])
        }
    }
}
