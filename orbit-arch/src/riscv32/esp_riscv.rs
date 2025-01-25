use esp_riscv_rt::{riscv::register::mcause, TrapFrame};

#[no_mangle]
static _max_hart_id: u32 = 0;
#[no_mangle]
extern "C" fn __post_init() {}
#[no_mangle]
extern "C" fn _setup_interrupts() {}

#[link_section = ".trap.rust"]
#[unsafe(export_name = "_start_trap_rust_hal")]
pub unsafe extern "C" fn start_trap_rust_hal(trap_frame: *mut TrapFrame) {
    assert!(
        mcause::read().is_exception(),
        "Arrived into _start_trap_rust_hal but mcause is not an exception!"
    );
    extern "C" {
        fn ExceptionHandler(tf: *mut TrapFrame);
    }
    ExceptionHandler(trap_frame);
}
