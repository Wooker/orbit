pub mod interrupt_controller;
pub mod port;

#[repr(u8)]
pub enum SystemId {
    InterruptController = 0,
    Port,
    MAX,
}
