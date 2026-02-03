#[repr(usize)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum SysCall {
    ReturnInit,
    ReturnMain,
    ReturnInterrupt,
    Delay,
    NumPorts,
    Send,
    SendAll,
    Await,
    ReceiveAll,
    MemAlloc,
    ClaimPeripheral,
    Invoke,
    Unknown = 0xff,
}

impl SysCall {
    pub const fn discriminant(&self) -> usize {
        unsafe { *(self as *const Self as *const usize) }
    }

    pub const fn from_usize(v: usize) -> Self {
        unsafe { *(&v as *const usize as *const Self) }
    }
}
