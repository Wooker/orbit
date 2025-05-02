#![no_std]
#![no_main]

use orbit_app::{blinky_x035::Blinky, calc::Calc, system_num_ports::SystemNumPorts};
use orbit_bin::orbit_main;

orbit_main!(Calc, Blinky, SystemNumPorts);
