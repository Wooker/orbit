#![no_std]
#![no_main]

use calc::Calc;
use orbit_bin::orbit_main;
use reade::Reade;

orbit_main!(Calc, Reade);
