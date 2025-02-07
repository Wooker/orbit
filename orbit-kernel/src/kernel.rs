use crate::clock::ClockConfig;

use chip::pac::Peripherals;
use orbit_arch::interface::timer::Timer;
use orbit_arch::Core;

use core::mem::MaybeUninit;
use fugit::HertzU32;
pub use fugit::{Rate, RateExtU32};

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
pub static KERNEL: Kernel = Kernel::new();

pub struct Kernel {
    pub peripherals: MaybeUninit<Peripherals>,
    // pub core: Core,
    pub apps: MaybeUninit<[u32; 8]>,
}
unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new() -> Self {
        Self {
            peripherals: { MaybeUninit::<Peripherals>::uninit() },
            // core: Core::new(hz),
            apps: MaybeUninit::uninit(),
        }
    }

    pub fn initialize(&mut self) -> ! {
        self.peripherals.write(unsafe { Peripherals::steal() });
        let peripherals = unsafe { self.peripherals.assume_init_mut() };

        let mut rcc = &peripherals.RCC;
        rcc.apb2prstr.write(|w| unsafe { w.bits(1 << 3) });
        rcc.apb2prstr
            .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << 3)) });

        rcc.apb2pcenr.write(|w| unsafe { w.bits((1 << 3)) });

        let mut gpiob = &peripherals.GPIOB;

        gpiob.cfghr.write(|w| unsafe { w.bits(0b0101) });
        gpiob.bshr.write(|w| unsafe { w.bits(1 << 24) });

        loop {
            gpiob.bshr.write(|w| unsafe { w.bits(1 << 8) });
            orbit_arch::riscv::asm::delay(1000000);
            gpiob.bshr.write(|w| unsafe { w.bits(1 << 24) });
            orbit_arch::riscv::asm::delay(1000000);
        }
    }

    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}

#[cfg(feature = "esp32c3")]
use orbit_arch::riscv::register::mcause;

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
