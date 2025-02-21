use crate::interface::pmp::Pmp;
use riscv::register::{pmpaddr0, pmpaddr1, pmpaddr2, pmpaddr3, pmpcfg0, Permission, Range};
use riscv::result;

pub struct Core<const PMP: usize> {
    pub pmp: RiscvPmp<PMP>,
}
impl<const PMP: usize> Core<PMP> {
    pub const fn new() -> Self {
        Self { pmp: RiscvPmp {} }
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
        match reg {
            0 => Ok(pmpaddr0::write(addr)),
            1 => Ok(pmpaddr1::write(addr)),
            2 => Ok(pmpaddr2::write(addr)),
            3 => Ok(pmpaddr3::write(addr)),
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
