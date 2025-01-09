use orbit_arch::arch::qingke::qingke_v4::Core;

#[used]
#[link_section = ".kernel"]
static KERNEL_MAJOR: u8 = 0;

#[used]
#[link_section = ".kernel"]
static KERNEL_MINOR: u8 = 1;

#[used]
#[link_section = ".kernel"]
pub static KERNEL: Kernel = Kernel::new();

use chips::ch32v208wbu6::Peripherals;

pub struct Kernel {
    core: Core,
}

unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new() -> Self {
        Self { core: Core::new() }
    }

    pub fn initialize(&self) -> Peripherals {
        unsafe { Peripherals::steal() }
    }
}
