#![no_std]
#![no_main]

use orbit_app::{blinky_v208::Blinky, calc::Calc, wfi::Wfi};
use orbit_bin::orbit_main;

orbit_main!(Calc, Blinky, Wfi);
