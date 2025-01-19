use core::arch::global_asm;

pub use esp32c3::*;

global_asm!(
    "
_abs_start_2:
    csrw mstatus, 0x0;
    "
);
