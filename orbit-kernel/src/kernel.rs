use core::{ptr::null_mut, sync::atomic::AtomicPtr};

#[cfg(feature = "ch32v208wbu6")]
use crate::peripherals::{
    gpio::{GPIOA, GPIOB},
    rcc::RCC,
};
#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static KERNEL_MAJOR: u8 = 0;
#[used]
#[no_mangle]
// #[link_section = ".kernel"]
pub static KERNEL_MINOR: u8 = 1;

// #[used]
// #[no_mangle]
// #[link_section = ".kernel"]
// pub static KERNEL: Kernel = Kernel::new();

#[cfg(feature = "ch32v208wbu6")]
use chip::Peripherals;

#[cfg(feature = "ch592")]
use chip::{pac::Peripherals, Reg, RegisterSpec};
#[cfg(feature = "ch592")]
use orbit_arch::Core;

#[cfg(feature = "esp32c3")]
use chip::Peripherals;

pub struct Kernel {
    pub peripherals: Peripherals,
    pub core: Core,
}

impl Kernel {
    pub fn new(hz: u32) -> Self {
        Self {
            peripherals: unsafe { Peripherals::steal() },
            core: Core::new(hz),
        }
    }

    pub fn claim<'a, P>(&self) -> *mut u32
    where
        P: RegisterSpec,
    {
        self.peripherals.GPIO.pa_dir.as_ptr()
    }

    #[cfg(feature = "ch32v208wbu6")]
    pub fn initialize(&self) -> ! {
        unsafe { orbit_arch::riscv32::riscv::register::mstatus::set_mie() };
        let mstatus = orbit_arch::riscv32::riscv::register::mstatus::read();
        let apb2_bits = (1 << 2) + (1 << 3) + (1 << 14); // PA, PB, USART1
        let rcc = RCC::new(&self.peripherals.RCC);
        rcc.reset(apb2_bits);
        rcc.enable_clock(apb2_bits);

        let gpioa = GPIOA::new(&self.peripherals.GPIOA);
        gpioa.enable();
        if mstatus.mie() {
            let gpiob = GPIOB::new(&self.peripherals.GPIOB);
            gpiob.enable();
        } else {
        }

        loop {
            // self.peripherals
            //     .GPIOB
            //     .bshr
            //     .write(|w| unsafe { w.bits(1 << 8) });
            // orbit_arch::riscv32::riscv::asm::delay(1000000);
            // self.peripherals
            //     .GPIOB
            //     .bshr
            //     .write(|w| unsafe { w.bits(1 << 24) });
            // orbit_arch::riscv32::riscv::asm::delay(1000000);
        }
    }

    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }
}
