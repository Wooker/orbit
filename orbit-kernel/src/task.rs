use crate::context::Context;

struct Task {
    pid: u32,
    context: Context,
    // pmp: [PmpEntry; PMP_REGS],
    x_addr: usize,
}
