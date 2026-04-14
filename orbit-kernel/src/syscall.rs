#[repr(usize)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum SysCall {
    /// Only used during driver initialization
    ReturnInit,
    /// Only used after application execution
    ReturnMain,
    /// Only used after driver interrupt handling
    ReturnInterrupt,
    /// Register interrupt for driver
    RegisterInterrupt,
    /// Request delay from kernel
    Delay,
    /// Request the number of kernel ports
    NumPorts,
    /// Send a packet via specific kernel port
    Send,
    /// Send a packet via all kernel ports
    SendAll,
    /// Await the reply to a packet sent earlier via specific kernel portt
    Await,
    /// Await all replies to packets sent earlier via all kernel ports
    AwaitAll,
    /// Allocate some memory on the application heap
    MemAlloc,
    /// Only used during driver initialization to claim specific peripheral
    ClaimPeripheral,
    /// Invoke application
    Invoke,
    /// Invoke application locally
    InvokeLocal,
}

impl SysCall {
    pub const fn discriminant(&self) -> usize {
        unsafe { *(self as *const Self as *const usize) }
    }

    pub const fn from_usize(v: usize) -> Self {
        unsafe { *(&v as *const usize as *const Self) }
    }
}
