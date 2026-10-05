#[repr(usize)]
#[derive(Clone, PartialEq, Eq, Copy)]
pub enum SysCall {
    /// Only used during driver initialization
    ReturnInit,
    /// Only used after application execution
    ReturnMain,
    /// Only used after driver interrupt handling
    ReturnInterrupt,
    /// Only used after System execution
    ReturnSystem,
    /// Register interrupt for driver
    RegisterInterrupt,
    /// Register system for driver
    RegisterSystem,
    /// Handle system
    HandleSystem,
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

/// This enum shows the reason a [`SysCall::InvokeLocal`] request failed.
///
/// This is an ABI value: zero is reserved for success, while these values are
/// returned in `a0` to the caller of `InvokeLocal`.
#[repr(usize)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InvokeLocalError {
    ApplicationNotFound = 1,
    TaskCreationFailed = 2,
    Unknown = 3,
}

impl InvokeLocalError {
    pub const fn discriminant(self) -> usize {
        self as usize
    }

    pub const fn from_discriminant(value: usize) -> Option<Self> {
        match value {
            1 => Some(Self::ApplicationNotFound),
            2 => Some(Self::TaskCreationFailed),
            3 => Some(Self::Unknown),
            _ => None,
        }
    }
}
