#![no_std]
#![no_main]

use calc::Calc;
use orbit_bin::orbit_main;
use reade::Reade;
use spi_app::SpiApp;

orbit_main!(Calc, Reade);
