use crate::multiplexer::Multiplexer;
use crate::resources::Resources;

use esp_hal::peripherals::Peripherals as HALPeripherals;
use esp_hal::prelude::*;
use esp_hal::system::SystemParts;

/// Main communication layer between HALs and LibOSes
pub struct Kernel<'k> {
    pub multiplexer: Multiplexer,
    system: SystemParts<'k>,
}

// static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
// static mut HEAP: core::mem::MaybeUninit<[u8; 1024]> = core::mem::MaybeUninit::uninit();

impl<'k> Kernel<'k> {
    pub fn new() -> Self {
        let peripherals = HALPeripherals::take();
        let system = peripherals.SYSTEM.split();

        let resources = Resources {
            AES: peripherals.AES,
            DMA: peripherals.DMA,
        };

        let multiplexer = Multiplexer::new(resources);

        Self {
            multiplexer,
            system,
        }
    }
}

#[allow(dead_code)]
enum KernelError {}
