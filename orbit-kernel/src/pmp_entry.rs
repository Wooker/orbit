use orbit_arch::{riscv::register::Permission, riscv::register::Range};

#[derive(Clone, Copy)]
pub struct PmpEntry {
    pub address: usize,
    pub range: Range,
    pub permission: Permission,
    pub locked: bool,
}

// NAPOT: addr >> 2 | ((1<<(pow-3)-1)
// OTHER: addr >> 2
impl PmpEntry {
    #[allow(unused)]
    pub fn new(addr: usize, range: Range, permission: Permission, locked: bool) -> Self {
        Self {
            address: addr,
            range,
            permission,
            locked,
        }
    }
    pub const fn default() -> Self {
        Self {
            address: 0x0,
            range: Range::OFF,
            permission: Permission::NONE,
            locked: false,
        }
    }
}
