use crate::{
    application::{AppContainer, PmpEntry},
    clock::Clocks,
};
use chip::pac::Peripherals;
use fugit::HertzU32;
use orbit_arch::{interface::pmp::Pmp, riscv::register::Permission, riscv::register::Range, Core};

use core::arch::naked_asm;
use core::{arch::asm, mem::MaybeUninit};

#[used]
#[no_mangle]
#[link_section = ".kernel.rodata"]
pub static KERNEL_MAJOR: u8 = 0;

#[used]
#[no_mangle]
#[link_section = ".kernel.rodata"]
pub static KERNEL_MINOR: u8 = 1;

#[cfg(feature = "ch32v208wbu6")]
#[used]
#[no_mangle]
#[link_section = ".kernel.bss"]
pub static mut KERNEL: Kernel<4> = Kernel::new();

#[cfg(feature = "ch32v003")]
#[used]
#[no_mangle]
#[link_section = ".kernel.bss"]
pub static mut KERNEL: Kernel<0> = Kernel::new();

pub struct Kernel<const PMP: usize> {
    pub(crate) peripherals: MaybeUninit<Peripherals>,
    pub core: Core<PMP>,
    apps: [MaybeUninit<AppContainer<PMP>>; 4],
    pub clock: Clocks,
}
unsafe impl<const PMP: usize> Sync for Kernel<PMP> {}

impl<const PMP: usize> Kernel<PMP> {
    #[link_section = ".kernel.text"]
    pub const fn new() -> Self {
        Self {
            peripherals: { MaybeUninit::<Peripherals>::uninit() },
            core: Core::new(),
            apps: MaybeUninit::uninit_array::<4>(),
            clock: Clocks::default(),
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn add_application(
        &mut self,
        index: usize,
        app_struct: usize,
        app_main_addr: usize,
        stack: usize,
    ) {
        let app = unsafe { self.apps.get_unchecked_mut(index) };
        app.write(AppContainer::new(
            [PmpEntry::default(); PMP],
            app_struct,
            app_main_addr,
            stack,
        ));
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn initialize(&mut self) {
        // clock::ClockConfig::pll_60mhz().freeze();
        self.clock.freeze();
        self.peripherals.write(unsafe { Peripherals::steal() });
        self.core.pmp.default();

        self.context_switch(0);
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    fn context_switch(&mut self, index: usize) {
        let app_cont = unsafe { self.apps.get_unchecked(index).assume_init_read() };
        self.set_pmp(&app_cont);

        unsafe {
            let struct_p: usize = app_cont.struct_addr();
            let entry: usize = app_cont.main_addr();
            let stack_ptr: usize = app_cont.stack_addr();

            #[naked]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn switch() {
                naked_asm!("mv sp, t1", "mv a0, t2", "csrw mepc, t0", "mret");
            }
            core::arch::asm!(
                "mv t0, {0}", // Load entry into a0
                "mv t1, {1}", // Load stack pointer into a1
                "mv t2, {2}",
                "call {3}",   // Call the switch_to_app function
                in(reg) entry,
                in(reg) stack_ptr,
                in(reg) struct_p,
                sym switch
            );
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    fn set_pmp(&mut self, app: &AppContainer<PMP>) {
        for (i, pe) in app.get_pmp().iter().enumerate() {
            self.core
                .pmp
                .write_cfg(0, i, pe.range, pe.permission, pe.locked);
            self.core.pmp.write_addr(i, pe.address);
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn clock(&self) -> u32 {
        self.clock.hclk.raw()
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}
