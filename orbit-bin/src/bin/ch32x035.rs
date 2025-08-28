#![no_std]
#![no_main]

use orbit_app::{
    blinky_x035::Blinky, calc::Calc, eink::Eink, reader::Reade, system_num_ports::SystemNumPorts,
};
use orbit_bin::orbit_main;

const A: usize = orbit_kernel::claim::KernelPeripherals::GPIOA as usize;
const_assert!(A == 0);
orbit_main!(Blinky, Calc, SystemNumPorts, Reade, Eink);
