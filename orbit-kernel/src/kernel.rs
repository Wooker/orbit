#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static KERNEL_MAJOR: u8 = 0;
#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static KERNEL_MINOR: u8 = 1;

#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static mut KERNEL: Kernel = Kernel::new();

#[cfg(feature = "ch32v208wbu6")]
use chip::{Peripherals, GPIOB};

#[cfg(feature = "ch592")]
use chip::Peripherals;

#[cfg(feature = "esp32c3")]
use chip::Peripherals;

#[repr(C)]
pub struct Kernel {
    // core: Core,
    pub peripherals: Option<Peripherals>,
}

unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new() -> Self {
        Self { peripherals: None }
    }

    pub unsafe fn initialize(&mut self) {
        self.peripherals = Peripherals::take();
    }
}
