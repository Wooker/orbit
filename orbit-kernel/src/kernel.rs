use crate::{
    application::{AppContainer, PmpEntry},
    clock,
};
use chip::pac::Peripherals;
use orbit_arch::{interface::pmp::Pmp, riscv::register::Permission, riscv::register::Range, Core};

use core::{arch::asm, mem::MaybeUninit};

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
pub static mut KERNEL: Kernel<4> = Kernel::new(10);

pub struct Kernel<const PMP: usize> {
    pub peripherals: MaybeUninit<Peripherals>,
    pub core: Core<PMP>,
    pub apps: [MaybeUninit<AppContainer<PMP>>; 4],
}
unsafe impl<const PMP: usize> Sync for Kernel<PMP> {}

impl<const PMP: usize> Kernel<PMP> {
    pub const fn new(hz: u32) -> Self {
        Self {
            peripherals: { MaybeUninit::<Peripherals>::uninit() },
            core: Core::new(hz),
            apps: MaybeUninit::uninit_array::<4>(),
        }
    }

    #[inline(never)]
    pub fn add_application(&mut self, index: usize, app_addr: usize) {
        unsafe {
            self.apps
                .get_unchecked_mut(index)
                .write(AppContainer::new([PmpEntry::default(); PMP], app_addr))
        };
    }

    pub fn initialize(&mut self) {
        clock::ClockConfig::pll_60mhz().freeze();
        self.peripherals.write(unsafe { Peripherals::steal() });
        self.core.pmp.default();

        self.context_switch(0);
    }

    fn context_switch(&mut self, index: usize) {
        let app_cont = unsafe { self.apps.get_unchecked(index).assume_init_read() };
        self.set_pmp(&app_cont);

        orbit_arch::riscv::register::mepc::write(app_cont.get_addr());
        unsafe {
            orbit_arch::riscv::register::mstatus::set_mpp(
                orbit_arch::riscv::register::mstatus::MPP::User,
            )
        };
        unsafe { asm!("mret") };
    }

    fn set_pmp(&mut self, app: &AppContainer<PMP>) {
        for (i, pe) in app.get_pmp().iter().enumerate() {
            self.core
                .pmp
                .write_cfg(0, i, pe.range, pe.permission, pe.locked);
            self.core.pmp.write_addr(i, pe.address);
        }
    }

    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}
