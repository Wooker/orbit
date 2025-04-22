pub enum SysCall {
    Delay,
    Claim,
    NumPorts,
    Send,
    SendAll,
    Unknown,
}

impl Into<usize> for SysCall {
    fn into(self) -> usize {
        match self {
            SysCall::Delay => 1,
            SysCall::Claim => 2,
            SysCall::NumPorts => 3,
            SysCall::Send => 4,
            SysCall::SendAll => 5,
            SysCall::Unknown => usize::MAX,
        }
    }
}

impl From<usize> for SysCall {
    fn from(value: usize) -> Self {
        match value {
            1 => SysCall::Delay,
            2 => SysCall::Claim,
            3 => SysCall::NumPorts,
            4 => SysCall::Send,
            5 => SysCall::SendAll,
            _ => SysCall::Unknown,
        }
    }
}
