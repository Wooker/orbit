use crate::interface::{pmp::Pmp, timer::Timer};
use embedded_hal::delay::DelayNs;
use qingke::riscv::{
    asm,
    delay::McycleDelay,
    register::{
        mhpmcounter5h::write,
        pmpcfg0::{self, clear_pmp},
        Permission, Range,
    },
    result::{Error as RiscvError, Result},
};

pub struct Core {
    pub pmp: CorePmp,
    pub timer: CoreTimer,
}
impl Core {
    pub fn new(hz: u32) -> Self {
        Self {
            pmp: CorePmp {},
            timer: CoreTimer::new(hz),
        }
    }
}

pub struct CorePmp;
impl Pmp<Permission, Range> for CorePmp {
    type Result<T> = Result<T>;

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
            _ => Err(RiscvError::IndexOutOfBounds {
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
                0..=3 => Ok(unsafe { clear_pmp(index) }),
                _ => Err(RiscvError::IndexOutOfBounds {
                    index,
                    min: 0,
                    max: 3,
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
            0 => Ok(qingke::riscv::register::pmpaddr0::write(addr)),
            1 => Ok(qingke::riscv::register::pmpaddr1::write(addr)),
            2 => Ok(qingke::riscv::register::pmpaddr2::write(addr)),
            3 => Ok(qingke::riscv::register::pmpaddr3::write(addr)),
            _ => Err(RiscvError::IndexOutOfBounds {
                index: reg,
                min: 0,
                max: 3,
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

pub struct CoreTimer {
    hz: u32,
}
impl CoreTimer {
    fn new(hz: u32) -> Self {
        Self { hz }
    }
}
impl Timer for CoreTimer {
    /// Delay in nanoseconds. Arguments shows the minimum amount as the
    /// operation may take longer time
    fn delay(&self, ns: u32) {
        if let Some(duration) = self.hz.checked_div_euclid(ns) {
            asm::delay(duration)
        } else {
        }
    }
}
