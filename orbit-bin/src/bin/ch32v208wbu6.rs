#![no_std]
#![no_main]

use orbit_app::{application::Application, blinky_v208::Blinky, wfi::Wfi};
use orbit_bin::orbit_main;

orbit_main!(Blinky, Wfi);
