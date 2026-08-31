use crate::interface::{pmp::Pmp, timer::Timer, Interruptable, SetupMtvec};
use core::arch::global_asm;
use embedded_hal::delay::DelayNs;
use qingke::riscv::{
    asm,
    delay::McycleDelay,
    register::{
        mhpmcounter5h::write,
        mtvec,
        pmpcfg0::{self, clear_pmp},
        Permission, Range,
    },
    result::{Error as RiscvError, Result},
};

pub use qingke::riscv;
pub use qingke::riscv::asm::delay;

#[cfg(feature = "qingke_v4")]
pub const PMP: usize = 4;

#[cfg(feature = "qingke_v2")]
pub const PMP: usize = 0;

pub struct Core<const PMP: usize> {
    pub pmp: RiscvPmp<PMP>,
    pub timer: CoreClock,
}
impl<const PMP: usize> Core<PMP> {
    pub fn new() -> Self {
        let mut pmp = RiscvPmp {};
        pmp.default();
        Self {
            pmp,
            timer: CoreClock::new(10),
        }
    }
}

pub struct RiscvPmp<const PMP: usize>;
impl<const PMP: usize> Pmp<Permission, Range> for RiscvPmp<PMP> {
    type Result<T> = qingke::riscv::result::Result<T>;

    fn write_cfg(
        &self,
        reg: usize,
        index: usize,
        r: Range,
        p: Permission,
        locked: bool,
    ) -> Self::Result<()> {
        match reg {
            0 => unsafe { qingke::riscv::register::pmpcfg0::try_set_pmp(index, r, p, locked) },
            _ => Err(RiscvError::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 0,
            }),
        }
    }

    fn read_cfg(&self, reg: usize, index: usize) -> Self::Result<usize> {
        match reg {
            0 => match qingke::riscv::register::pmpcfg0::try_read() {
                Ok(csr) => Ok(csr.bits),
                Err(e) => Err(e),
            },
            _ => Err(RiscvError::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 0,
            }),
        }
    }

    fn clear_cfg(&self, reg: usize, index: usize) -> Self::Result<()> {
        match reg {
            0 => match index {
                0..=3 => Ok(unsafe { qingke::riscv::register::pmpcfg0::clear_pmp(index) }),
                _ => Err(RiscvError::IndexOutOfBounds {
                    index,
                    min: 0,
                    max: PMP - 1,
                }),
            },
            _ => Err(RiscvError::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 0,
            }),
        }
    }

    fn write_addr(&self, reg: usize, addr: usize) -> Self::Result<()> {
        match reg {
            0 => unsafe { Ok(qingke::riscv::register::pmpaddr0::write(addr)) },
            1 => unsafe { Ok(qingke::riscv::register::pmpaddr1::write(addr)) },
            2 => unsafe { Ok(qingke::riscv::register::pmpaddr2::write(addr)) },
            3 => unsafe { Ok(qingke::riscv::register::pmpaddr3::write(addr)) },
            _ => Err(RiscvError::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: PMP - 1,
            }),
        }
    }

    fn read_addr(&self, reg: usize, addr: usize) -> Self::Result<usize> {
        match reg {
            0 => Ok(qingke::riscv::register::pmpaddr0::read()),
            1 => Ok(qingke::riscv::register::pmpaddr1::read()),
            2 => Ok(qingke::riscv::register::pmpaddr2::read()),
            3 => Ok(qingke::riscv::register::pmpaddr3::read()),
            _ => Err(RiscvError::IndexOutOfBounds {
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

enum SysClockMode {
    Up,
    Down,
}
enum SysClockSource {
    HCLK,
    HCLK8Division,
}
impl CoreClock {
    const fn new(hz: u32) -> Self {
        Self { hz }
    }

    fn configure(
        &mut self,
        sw_int_en: bool,
        int_en: bool,
        mode: SysClockMode,
        source: SysClockSource,
    ) {
    }
    fn start(&mut self) {}
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
    fn enable_interrupt(&self, interrupt_code: u8) {
        unsafe { qingke::pfic::enable_interrupt(interrupt_code) };
    }
    fn disable_interrupt(&self, interrupt_code: u8) {
        unsafe { qingke::pfic::disable_interrupt(interrupt_code) };
    }
}

impl<const PMP: usize> SetupMtvec for Core<PMP> {
    type TrapMode = mtvec::TrapMode;

    fn setup_mtvec(&self, addr: usize, mode: Self::TrapMode) {
        unsafe { mtvec::write(mtvec::Mtvec::new(addr, mode)) };
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
    // Set dcsr 9 and 11 bits
    "
    csrr t0, dcsr;
    li t1, 0xa00;
    or t0, t0, t1;
    csrw dcsr, t0;
    ",
    // Jump to main
    "
    la t0, main;
    csrw mepc, t0;
    mret;
    "
);
