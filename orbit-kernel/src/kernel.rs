use crate::clock;
use chip::pac::Peripherals;
use core::sync::atomic::{compiler_fence, fence, Ordering};
use orbit_arch::{interface::pmp::Pmp, riscv::register::Permission, riscv::register::Range, Core};

use core::{arch::asm, mem::MaybeUninit};
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
pub static mut KERNEL: Kernel = Kernel::new(10);

pub struct Kernel {
    pub peripherals: MaybeUninit<Peripherals>,
    pub core: Core,
    pub apps: [MaybeUninit<usize>; 4],
}
unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new(hz: u32) -> Self {
        Self {
            peripherals: { MaybeUninit::<Peripherals>::uninit() },
            core: Core::new(hz),
            apps: MaybeUninit::uninit_array::<4>(),
        }
    }

    pub fn register(&mut self, app: usize) {
        self.apps[0].write(app);
        fence(Ordering::SeqCst);
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
            .write_cfg(0, 0, Range::TOR, Permission::RX, false);
        self.core.pmp.write_addr(0, 0x0800_0000 >> 2);
        let app_addr = unsafe { self.apps[0].assume_init_read() };
        if app_addr > 0 {
            orbit_arch::riscv::register::mepc::write(app_addr);
            unsafe {
                orbit_arch::riscv::register::mstatus::set_mpp(
                    orbit_arch::riscv::register::mstatus::MPP::User,
                )
            };
            unsafe { asm!("mret") };
        }
    }

    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}
