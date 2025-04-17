#![no_std]
#![no_main]

use orbit_app::{blinky_x035::Blinky, calc::Calc};
use orbit_bin::orbit_main;

orbit_main!(Calc, Blinky);
