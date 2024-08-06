//! Orbit

#![no_std]
#![no_main]

use core::{arch::asm, marker::PhantomData, panic::PanicInfo};

#[cfg(feature = "esp32c3")]
use esp32c3::Peripherals;
#[cfg(feature = "esp32c3")]
use esp_riscv_rt;
// use esp_alloc::EspHeap;
// pub use esp_backtrace;

// #[cfg(feature = "stm32f103")]
// use panic_halt as _;

#[cfg(feature = "stm32f103")]
pub use cortex_m_rt::entry;
#[cfg(feature = "stm32f103")]
pub use stm32f1::stm32f103::Peripherals;

// static HEAP: EspHeap = EspHeap::empty();

pub struct Kernel<'k> {
    multiplexer: Multiplexer<'k>,
}
impl<'k> Kernel<'k> {
    pub fn new() -> Self {
        let peripherals = unsafe { Peripherals::steal() };
        let multiplexer = Multiplexer::new(peripherals);

        Self { multiplexer }
    }
}

pub struct Multiplexer<'m> {
    peripherals: Peripherals,
    table: ResourceTable<15>,
    _m: PhantomData<&'m bool>,
}

impl<'m> Multiplexer<'m> {
    fn new(peripherals: Peripherals) -> Self {
        Self {
            peripherals,
            table: ResourceTable::new(),
            _m: PhantomData,
        }
    }
}

struct ResourceTable<const N: usize> {
    count: usize,
    list: [bool; N],
}

impl<const N: usize> ResourceTable<N> {
    fn new() -> Self {
        Self {
            count: N,
            list: [false; N],
        }
    }
}

#[panic_handler]
unsafe fn panic<'a, 'b>(_info: &'a PanicInfo<'b>) -> ! {
    loop {
        asm!("nop")
    }
}
