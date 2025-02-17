use chip::pac::Peripherals;
use orbit_arch::{interface::pmp::Pmp, riscv::register::Permission, riscv::register::Range, Core};

use core::mem::MaybeUninit;
pub use fugit::{Rate, RateExtU32};

extern "C" {
    static _sapps: u8;
    static _eapps: u8;
}

#[used]
#[no_mangle]
#[link_section = ".kernel"]
pub static KERNEL_MAJOR: u8 = 0;

#[used]
#[no_mangle]
#[link_section = ".kernel"]
pub static KERNEL_MINOR: u8 = 1;

#[used]
#[no_mangle]
#[link_section = ".kernel"]
pub static KERNEL: Kernel = Kernel::new(10);

pub struct Kernel {
    pub peripherals: MaybeUninit<Peripherals>,
    pub core: Core,
    pub apps: MaybeUninit<[u32; 8]>,
}
unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new(hz: u32) -> Self {
        Self {
            peripherals: { MaybeUninit::<Peripherals>::uninit() },
            core: Core::new(hz),
            apps: MaybeUninit::uninit(),
        }
    }

    pub fn initialize(&mut self) {
        clock::ClockConfig::pll_60mhz().freeze();
        self.peripherals.write(unsafe { Peripherals::steal() });
        self.core.pmp.clear_cfg(0, 0);
        self.core.pmp.clear_cfg(0, 1);
        self.core.pmp.clear_cfg(0, 2);
        self.core.pmp.clear_cfg(0, 3);
        self.core
            .pmp
            .write_cfg(0, 3, Range::TOR, Permission::NONE, false);
        self.core.pmp.write_addr(3, 0x0);
        let apps_size = unsafe { _eapps - _sapps };
    }

    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}

#[cfg(feature = "esp32c3")]
use orbit_arch::riscv::register::mcause;

use crate::clock;

#[cfg(feature = "esp32c3")]
#[no_mangle]
#[unsafe(link_section = ".trap")]
pub(super) unsafe extern "C" fn _handle_priority() -> u32 {
    let interrupt_id: usize = mcause::read().code(); // MSB is whether its exception or interrupt.
    let intr = &*chip::INTERRUPT_CORE0::PTR;
    let interrupt_priority = intr
        .cpu_int_pri(0)
        .as_ptr()
        .add(interrupt_id)
        .read_volatile();

    let prev_interrupt_priority = intr.cpu_int_thresh().read().bits();
    if interrupt_priority < 15 {
        // leave interrupts disabled if interrupt is of max priority.
        intr.cpu_int_thresh()
            .write(|w| w.bits(interrupt_priority + 1)); // set the prio threshold to 1 more than current interrupt prio
        unsafe {
            orbit_arch::riscv::interrupt::enable();
        }
    }
    prev_interrupt_priority
}

#[cfg(feature = "esp32c3")]
#[no_mangle]
#[unsafe(link_section = ".trap")]
pub(super) unsafe extern "C" fn _restore_priority(stored_prio: u32) {
    orbit_arch::riscv::interrupt::disable();
    let intr = &*chip::INTERRUPT_CORE0::PTR;
    intr.cpu_int_thresh().write(|w| w.bits(stored_prio));
}
