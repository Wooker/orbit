use crate::interface::{Interruptable, SetupMtvec, pmp::Pmp, timer::Timer};
use core::arch::global_asm;
use embedded_hal::delay::DelayNs;
pub use riscv::asm::delay;
use riscv::interrupt::InterruptNumber;
use riscv::register::{
    Permission, Range, mie, mtvec, pmpaddr0, pmpaddr1, pmpaddr2, pmpaddr3, pmpcfg0,
};
use riscv::{asm, result};

pub const PMP: usize = 4;

pub use riscv;

pub struct Core<const PMP: usize> {
    pub clock: CoreClock,
    pub pmp: RiscvPmp<PMP>,
}
impl<const PMP: usize> Core<PMP> {
    pub const fn new() -> Self {
        Self {
            pmp: RiscvPmp {},
            clock: CoreClock { hz: 10 },
        }
    }
}

pub struct RiscvPmp<const PMP: usize>;
impl<const PMP: usize> Pmp<Permission, Range> for RiscvPmp<PMP> {
    type Result<T> = result::Result<T>;

    fn write_cfg(
        &self,
        reg: usize,
        index: usize,
        r: Range,
        p: Permission,
        locked: bool,
    ) -> Self::Result<()> {
        match reg {
            0 => unsafe { pmpcfg0::try_set_pmp(index, r, p, locked) },
            _ => Err(result::Error::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 0,
            }),
        }
    }

    fn read_cfg(&self, reg: usize, index: usize) -> Self::Result<usize> {
        match reg {
            0 => match pmpcfg0::try_read() {
                Ok(csr) => Ok(csr.bits),
                Err(e) => Err(e),
            },
            _ => Err(result::Error::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 0,
            }),
        }
    }

    fn clear_cfg(&self, reg: usize, index: usize) -> Self::Result<()> {
        match reg {
            0 => match index {
                0..=3 => Ok(unsafe { pmpcfg0::clear_pmp(index) }),
                _ => Err(result::Error::IndexOutOfBounds {
                    index,
                    min: 0,
                    max: PMP - 1,
                }),
            },
            _ => Err(result::Error::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 0,
            }),
        }
    }

    fn write_addr(&self, reg: usize, addr: usize) -> Self::Result<()> {
        // SAFETY: this code is guaranteed to run only on Risc-V targets
        match reg {
            0 => unsafe { Ok(pmpaddr0::write(addr)) },
            1 => unsafe { Ok(pmpaddr1::write(addr)) },
            2 => unsafe { Ok(pmpaddr2::write(addr)) },
            3 => unsafe { Ok(pmpaddr3::write(addr)) },
            _ => Err(result::Error::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: PMP - 1,
            }),
        }
    }

    fn read_addr(&self, reg: usize, addr: usize) -> Self::Result<usize> {
        match reg {
            0 => Ok(pmpaddr0::read()),
            1 => Ok(pmpaddr1::read()),
            2 => Ok(pmpaddr2::read()),
            3 => Ok(pmpaddr3::read()),
            _ => Err(result::Error::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 3,
            }),
        }
    }
}
impl<const PMP: usize> RiscvPmp<PMP> {
    pub fn default(&mut self) {
        for pmpcfg in 0..PMP {
            self.clear_cfg(0, pmpcfg);
            self.write_addr(pmpcfg, 0);
        }
    }
}

pub struct CoreClock {
    hz: u32,
}

impl Timer for CoreClock {
    /// Delay in nanoseconds. Arguments shows the minimum amount as the
    /// operation may take longer time
    #[inline(always)]
    fn delay(&self, ns: u32) {
        asm::delay(ns)
    }
}

impl<const PMP: usize> Interruptable for Core<PMP> {
    fn enable_interrupt(&self, interrupt_code: usize) {
        if let Ok(irq) = riscv::interrupt::Interrupt::from_number(interrupt_code) {
            unsafe { mie::enable(irq) };
        }
    }
    fn disable_interrupt(&self, interrupt_code: usize) {
        if let Ok(irq) = riscv::interrupt::Interrupt::from_number(interrupt_code) {
            unsafe { mie::disable(irq) };
        }
    }
}

impl<const PMP: usize> SetupMtvec for Core<PMP> {
    type TrapMode = mtvec::TrapMode;

    fn setup_mtvec(&self, addr: usize, mode: Self::TrapMode) {
        unsafe { mtvec::write(mtvec::Mtvec::new(addr, mode)) }
    }
}

global_asm!(
    "
    .section .init
    .global _start

_start:
    la sp, _stack_top
    li t0, 0x1880
    csrw mstatus, t0
    ",
    // Set mtvec
    "
    la t0, handler;
    csrw mtvec, t0;
    ",
    #[cfg(feature = "ext-ints")]
    "
    li      t0, 0x800
    csrs    mie, t0  
    ",
    // Setup PMP
    "
    li t0, 0
    csrw pmpaddr0, t0
    li t0, 0xffffffff
    srli t0, t0, 2
    csrw pmpaddr1, t0
    li t0, 0x00000F00
    csrw pmpcfg0, t0
    ",
    // Jump to main
    "
    la t0, main;
    csrw mepc, t0;
    mret;
    "
);
