//! Orbit

#![no_std]
#![no_main]

use core::marker::PhantomData;

use esp32c3::Peripherals;
use esp_alloc::EspHeap;
pub use esp_backtrace;

static HEAP: EspHeap = EspHeap::empty();

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
