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
struct SpiApp {}

#[orbit_impl]
impl SpiApp {
    #[app_init]
    pub fn init(&mut self) {}

    #[app_interrupt]
    pub fn interrupt(&mut self) {}

    #[app_main]
    pub fn main(&mut self) {
        let mut gpioa = Self::claim_peripheral_gpioa();
        let mut c_gp = Claimed::new(&mut gpioa);
        let mut spi1 = Self::claim_peripheral_spi1();
        let mut c_spi = Claimed::new(&mut spi1);
        let bus = Spi::new(&mut c_spi, Config::default());
        let eink: LibEink<DC_PIN, BUSY_PIN> = LibEink::new(bus, &mut c_gp);
    }
}
