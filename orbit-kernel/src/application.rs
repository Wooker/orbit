use orbit_arch::{riscv::register::Permission, riscv::register::Range, Core};

#[derive(Clone, Copy)]
pub(crate) struct PmpEntry {
    pub address: usize,
    pub range: Range,
    pub permission: Permission,
    pub locked: bool,
}

// NAPOT: addr >> 2 | ((1<<(pow-3)-1)
// OTHER: addr >> 2
impl PmpEntry {
    pub fn new(addr: usize, range: Range, permission: Permission, locked: bool) -> Self {
        Self {
            address: addr,
            range,
            permission,
            locked,
        }
    }
}
impl Default for PmpEntry {
    fn default() -> Self {
        Self {
            address: 0x0,
            range: Range::OFF,
            permission: Permission::NONE,
            locked: false,
        }
    }
}

#[derive(Clone, Copy)]
pub struct AppContainer<const PMP_REGS: usize> {
    pmp: [PmpEntry; PMP_REGS],
    app_struct: usize,
    app_main_addr: usize,
    app_stack_addr: usize,
}

impl<const PMP_REGS: usize> AppContainer<PMP_REGS> {
    pub fn new(
        pmp: [PmpEntry; PMP_REGS],
        app_struct: usize,
        app_main_addr: usize,
        app_stack_addr: usize,
    ) -> Self {
        Self {
            pmp,
            app_struct,
            app_main_addr,
            app_stack_addr,
        }
    }

    pub fn struct_addr(&self) -> usize {
        self.app_struct
    }

    pub fn main_addr(&self) -> usize {
        self.app_main_addr
    }

    pub fn stack_addr(&self) -> usize {
        self.app_stack_addr
    }

    pub fn get_pmp(&self) -> [PmpEntry; PMP_REGS] {
        self.pmp
    }
}
