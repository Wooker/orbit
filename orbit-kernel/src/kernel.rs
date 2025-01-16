#[used]
#[link_section = ".kernel"]
static KERNEL_MAJOR: u8 = 0;

#[used]
#[link_section = ".kernel"]
static KERNEL_MINOR: u8 = 1;

#[used]
#[link_section = ".kernel"]
pub static KERNEL: Kernel = Kernel::new();

#[cfg(feature = "ch32v208wbu6")]
use chips::ch32v208wbu6::Peripherals;

#[cfg(feature = "ch592")]
use chips::ch592::Peripherals;

#[cfg(feature = "esp32c3")]
use chips::esp32c3::Peripherals;

pub struct Kernel {
    // core: Core,
}

unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new() -> Self {
        Self {/* core: Core::new() */}
    }

    pub fn initialize(&self) -> Peripherals {
        unsafe { Peripherals::steal() }
    }
}
