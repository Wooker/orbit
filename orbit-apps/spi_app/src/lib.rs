#![no_std]

use ch32x035_spi_driver::{Config, Spi};
use eink_lib::Eink as LibEink;
use orbit_app_common::{app_heap, app_stack};
use orbit_app_proc_macro::{app_init, app_interrupt, app_main, orbit_app, orbit_impl};
use orbit_kernel::chip::pac::{GPIOA, SPI1};

const DC_PIN: u8 = 1;
const BUSY_PIN: u8 = 6;

app_heap!(32);
app_stack!(32);

#[orbit_app(SPI1, GPIOA)]
struct SpiApp;
#[orbit_impl]
impl SpiApp {
    #[app_init]
    pub fn init(&mut self) {}

    #[app_interrupt]
    pub fn interrupt(&mut self) {}

    #[app_main]
    pub fn main(_buf: &[u8], peripherals: &mut Peripherals) {
        let mut spi1 = unsafe { peripherals.spi1.assume_init_read() };
        let mut gpioa = unsafe { peripherals.gpioa.assume_init_read() };

        let bus = Spi::new(&mut spi1, Config::default());
        let _eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut gpioa);
    }
}
