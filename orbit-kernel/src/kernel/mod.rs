pub mod claim;

use chip::{
    pac::{GPIO, I2C, UART1},
    ClaimablePeripheral,
};
use claim::{Claim, ClaimError, Claimed};

use core::mem::MaybeUninit;

#[cfg(feature = "ch32v208wbu6")]
use crate::peripherals::{
    gpio::{GPIOA, GPIOB},
    rcc::RCC,
};
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
pub static KERNEL: Kernel = Kernel::new(32_000_000);

#[cfg(feature = "ch32v208wbu6")]
use chip::Peripherals;

#[cfg(feature = "ch592")]
use chip::pac::Peripherals;
#[cfg(feature = "ch592")]
use orbit_arch::Core;

#[cfg(feature = "esp32c3")]
use chip::Peripherals;

pub struct Kernel {
    pub peripherals: MaybeUninit<Peripherals>,
    pub core: Core,
    pub apps: MaybeUninit<[u32; 8]>,
}
unsafe impl Sync for Kernel {}

impl Kernel {
    pub const fn new(hz: u32) -> Self {
        Self {
            peripherals: { MaybeUninit::<Peripherals>::uninit() }, //Peripherals::steal() },
            core: Core::new(hz),
            apps: MaybeUninit::uninit(),
        }
    }

    #[cfg(feature = "ch592")]
    pub unsafe fn initialize(&mut self) {
        self.peripherals.write(Peripherals::steal());
        self.apps.write([1, 2, 3, 4, 5, 6, 7, 8]);
    }

    // pub fn claim(&mut self) -> Result<&chip::pac::UART3, ClaimError> {
    //     let peripherals: &mut Peripherals = unsafe { self.peripherals.assume_init_mut() };
    //     peripherals.claim()
    // }

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

impl<'p> Claim<'p, UART1> for Kernel {
    fn claim(&'p mut self, peripheral: ClaimablePeripheral) -> Result<Claimed<UART1>, ClaimError> {
        let peripherals = unsafe { self.peripherals.assume_init_mut() };
        match peripheral {
            ClaimablePeripheral::UART1 => {
                if 1 == 1 {
                    Ok(Claimed::new(&mut peripherals.UART1))
                } else {
                    Err(ClaimError::AlreadyClaimed)
                }
            }
            _ => Err(ClaimError::WrongType),
        }
    }
}

impl<'p> Claim<'p, I2C> for Kernel {
    fn claim(&'p mut self, peripheral: ClaimablePeripheral) -> Result<Claimed<I2C>, ClaimError> {
        let peripherals = unsafe { self.peripherals.assume_init_mut() };
        match peripheral {
            ClaimablePeripheral::I2C => {
                if 1 == 1 {
                    Ok(Claimed::new(&mut peripherals.I2C))
                } else {
                    Err(ClaimError::AlreadyClaimed)
                }
            }
            _ => Err(ClaimError::WrongType),
        }
    }
}

impl<'p> Claim<'p, GPIO> for Kernel {
    fn claim(&'p mut self, peripheral: ClaimablePeripheral) -> Result<Claimed<GPIO>, ClaimError> {
        let peripherals = unsafe { self.peripherals.assume_init_mut() };
        match peripheral {
            ClaimablePeripheral::GPIO => {
                if 1 == 1 {
                    Ok(Claimed::new(&mut peripherals.GPIO))
                } else {
                    Err(ClaimError::AlreadyClaimed)
                }
            }
            _ => Err(ClaimError::WrongType),
        }
    }
}
