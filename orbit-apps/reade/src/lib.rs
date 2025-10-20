#![no_std]

use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

use ch32x035_spi_driver::{Config, Spi as LibSpi};
use eink_lib::{Config as EinkConfig, Direction, Eink as LibEink, Position, Size};
use font_8x10::{self, ASCII};

app_heap!(0);
app_stack!(1024);

const DC_PIN: u8 = 1;
const BUSY_PIN: u8 = 6;

const WIDTH: usize = 25;
const HEIGHT: usize = 20;
const LAYERS: usize = 10;

struct S {
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

#[orbit_app(GPIOA, SPI1)]
struct Reade;
#[orbit_impl]
impl Reade {
    #[app_init("reade")]
    fn init(&mut self) {}

    #[app_interrupt("reade")]
    fn interrupt(&mut self) {}

    #[app_main("reade")]
    fn main(
        ringbuf: &mut RingBuf<RINGBUF_SIZE, RingbufType>,
        peripherals: &mut Peripherals,
    ) -> Output {
        let mut spi1 = unsafe { peripherals.spi1.assume_init_read() };
        let mut gpioa = unsafe { peripherals.gpioa.assume_init_read() };
        let mut s = S {
            letters: RingBuf::default(),
            frame: RingBuf::default(),
        };

        let buf = unsafe { ringbuf.read().unwrap_unchecked() };
        if let Ok(arg) = str::from_utf8(buf) {
            let arg = arg.trim();
            let cmp = arg[..4].cmp("show");

            match cmp {
                core::cmp::Ordering::Equal => {
                    for line in 0..HEIGHT {
                        for layer in 0..LAYERS {
                            for ch in 0..WIDTH {
                                let pos = s.letters.buf[ch + (line * WIDTH)] as usize;
                                s.frame.buf[((layer * WIDTH) + ch) + (line * WIDTH * LAYERS)] =
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
                    let bus = LibSpi::new(&mut spi1, Config::default());
                    let mut eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut gpioa);
                    eink.display(config, &s.frame.buf, false);
                    s.letters.flush();
                    s.frame.flush();
                    Output([0])
                }
                _ => {
                    let chars = arg.chars();
                    chars.clone().for_each(|b| s.letters.push(b as u8));
                    Output([chars.count() as u8])
                }
            }
        } else {
            Output([1])
        }
    }
}
