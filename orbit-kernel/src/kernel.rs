use crate::{
    application::{AppContainer, PmpEntry},
    clock::Clocks,
};
use chip::pac::Peripherals;
use core::arch::naked_asm;
use core::{arch::asm, mem::MaybeUninit};
use fugit::HertzU32;
use orbit_arch::riscv::register::mtvec;
use orbit_arch::{interface::pmp::Pmp, riscv::register::Permission, riscv::register::Range, Core};

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

#[cfg(feature = "ch32v208wbu6")]
const VECTOR_TABLE_SIZE: usize = 103;

#[cfg(feature = "ch32v003")]
const VECTOR_TABLE_SIZE: usize = 38;

extern "C" {
    static _handler: usize;
}

#[used]
#[no_mangle]
#[link_section = ".kernel.bss"]
pub static mut VECTOR_TABLE: [usize; VECTOR_TABLE_SIZE] = [0; VECTOR_TABLE_SIZE];

pub struct Kernel<const PMP: usize> {
    pub(crate) peripherals: MaybeUninit<Peripherals>,
    pub core: Core<PMP>,
    apps: [MaybeUninit<AppContainer<PMP>>; 4],
    pub clock: Clocks,
    sp: usize,
}
unsafe impl<const PMP: usize> Sync for Kernel<PMP> {}

impl<const PMP: usize> Kernel<PMP> {
    #[link_section = ".kernel.text"]
    pub const fn new() -> Self {
        Self {
            sp: 0,
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
        // use crate::_VECTOR_TABLE_INTERRUPTS;
        // unsafe { _VECTOR_TABLE_INTERRUPTS[0] = 1 };
        self.handler();

        // unsafe {
        //     // RCC
        //     orbit_arch::pfic::enable_interrupt(21);
        //     orbit_arch::register::gintenr::set_enable();
        // }

        for i in 0..VECTOR_TABLE_SIZE {
            unsafe {
                VECTOR_TABLE[i] = &_handler as *const usize as usize;
            }
        }

        self.clock.freeze();
        self.peripherals.write(unsafe { Peripherals::steal() });
        self.core.pmp.default();
        self.sp = 0;

        unsafe {
            mtvec::write(
                &_handler as *const usize as usize,
                // unsafe { &VECTOR_TABLE as *const usize as usize },
                mtvec::TrapMode::Direct,
            );
        }

        unsafe {
            // UART4
            orbit_arch::pfic::enable_interrupt(68);
            orbit_arch::pfic::enable_vtf(3, 68, 0x20000000);
        }

        loop {
            self.context_switch(0);
            self.context_switch(1);
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    fn context_switch(&mut self, index: usize) {
        let app_cont = unsafe { self.apps.get_unchecked(index).assume_init_read() };
        self.set_pmp(&app_cont);

        unsafe {
            let entry: usize = app_cont.main_addr();
            let stack_ptr: usize = app_cont.stack_addr();
            let struct_p: usize = app_cont.struct_addr();

            // Store kernel stack pointer
            asm!("mv {0}, sp", out(reg) self.sp);

            asm!(
                "mv t0, {0}", // Load entry into a0
                "mv t1, {1}", // Load stack pointer into a1
                "mv t2, {2}", // Load pointer to app struct
                "call {3}",   // Call the switch function
                in(reg) entry,
                in(reg) stack_ptr,
                in(reg) struct_p,
                sym switch
            );

            #[naked]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn switch() {
                naked_asm!(
                    "mv sp, t1",
                    "mv a0, t2",
                    "csrwi mstatus, 0",
                    "csrw mepc, t0",
                    "mret",
                    // "jalr zero, t0, 0"
                );
            }

            // Restore kernel stack pointer
            asm!("mv sp, {0}", in(reg) self.sp);
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

    #[no_mangle]
    #[inline(never)]
    #[link_section = ".kernel.text.handler"]
    pub fn handler(&mut self) {
        let mcause = orbit_arch::riscv::register::mcause::read();
        if mcause.is_interrupt() {
            self.context_switch(1);
        } else {
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}
