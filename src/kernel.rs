use core::marker::PhantomData;
use core::time::Duration;

use crate::multiplexer::Multiplexer;
use crate::resources::Resources;

use esp_hal::clock::{ClockControl, Clocks, CpuClock};
use esp_hal::peripherals::Peripherals as HALPeripherals;
use esp_hal::system::{CpuControl, SystemParts};
use esp_hal::{prelude::*, Delay};

/// Main communication layer between HALs and LibOSes
pub struct Kernel<'k> {
    pub multiplexer: Multiplexer,
    clocks: Clocks<'k>,
    _ph: PhantomData<&'k bool>,
}

// static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
// static mut HEAP: core::mem::MaybeUninit<[u8; 1024]> = core::mem::MaybeUninit::uninit();

impl<'k> Kernel<'k> {
    pub fn new() -> Self {
        let peripherals = HALPeripherals::take();
        let system = peripherals.SYSTEM.split();
        let clock_control = system.clock_control;
        let a = system.cpu_control;
        let b = system.radio_clock_control;
        let c = system.software_interrupt_control;

        let clocks = ClockControl::max(clock_control).freeze();
        let resources = Resources {
            AES: peripherals.AES,
            DMA: peripherals.DMA,
            GPIO: peripherals.GPIO,
            IO_MUX: peripherals.IO_MUX,
        };

        let multiplexer = Multiplexer::new(resources);

        Self {
            multiplexer,
            clocks,
            _ph: PhantomData,
        }
    }

    pub fn delay(&self, d: Duration) {
        Delay::new(&self.clocks).delay_micros(d.as_micros() as u32)
    }
}

#[allow(dead_code)]
enum KernelError {}
