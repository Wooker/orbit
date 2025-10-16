#[repr(usize)]
#[derive(Clone, PartialEq, Eq)]
pub enum SysCall {
    Return,
    Delay = 1,
    NumPorts,
    Send,
    SendAll,
    Await,
    ReceiveAll,
    Unknown,
    ClaimPeripheral,
}

impl SysCall {
    pub const fn discriminant(&self) -> usize {
        unsafe { *(self as *const Self as *const usize) }
    }
}
