use crate::multiplexer::Multiplexer;
use esp_hal::prelude::*;

/// Main communication layer between HALs and LibOSes
pub struct Kernel {
    pub multiplexer: Multiplexer,
}

// static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
// static mut HEAP: core::mem::MaybeUninit<[u8; 1024]> = core::mem::MaybeUninit::uninit();

use esp_hal::peripherals::Peripherals as HALPeripherals;

impl Kernel {
    pub fn new() -> Self {
        let peripherals = HALPeripherals::take();
        // let system = peripherals.SYSTEM.split();
        let multiplexer = Multiplexer::new(&mut Some(peripherals));
        Self { multiplexer }
    }
}

#[allow(dead_code)]
enum KernelError {}
