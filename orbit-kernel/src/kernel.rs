/*
pub trait Kernel {
    fn init();
    fn claim<Peripherals>(peripherals: Peripherals);
    fn allocate<Resource>(proc: Proc, res: Resource);
}
*/

use orbit_arch::arch::qingke::qingke_v4::Core;

#[used]
#[link_section = ".kernel"]
pub(crate) static KERNEL: Kernel = Kernel::new();

pub struct Kernel {
    core: Core,
}

impl Kernel {
    pub const fn new() -> Self {
        Self { core: Core::new() }
    }

    pub fn initialize() {}
}
