use crate::arch::PrivilegeMode;

pub struct Core {
    mode: PrivilegeMode,
}

impl Core {
    pub const fn new() -> Self {
        Self {
            mode: PrivilegeMode::Machine,
        }
    }

    fn set_freq() {
        todo!()
    }

    fn init() {
        todo!()
    }
}
