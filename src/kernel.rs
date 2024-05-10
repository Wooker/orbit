use crate::multiplexer::Multiplexer;

/// Main communication layer between HALs and LibOSes
pub struct Kernel {
    pub multiplexer: Multiplexer,
}

// static ALLOCATOR: esp_alloc::EspHeap = esp_alloc::EspHeap::empty();
// static mut HEAP: core::mem::MaybeUninit<[u8; 1024]> = core::mem::MaybeUninit::uninit();

use esp_hal::peripherals::*;

impl Kernel {
    pub fn new() -> Self {
        let multiplexer = Multiplexer::new();
        Self { multiplexer }
    }
}

#[allow(dead_code)]
enum KernelError {}
