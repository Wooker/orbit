pub mod qingke;

use ::qingke::riscv::register;

enum PrivilegeMode {
    User,
    Supervisor,
    Machine,
}

struct CSR {}
impl CSR {
    fn new() -> Self {
        Self {}
    }
}
