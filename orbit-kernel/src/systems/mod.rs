pub mod external_interrupt_controller;
pub mod port;

#[repr(u8)]
pub enum SystemId {
    ExternalInterruptController = 0,
    Port,
    MAX,
}

#[repr(u8)]
pub enum SystemIdError {
    UnknownId = 0,
}

impl TryFrom<usize> for SystemId {
    type Error = SystemIdError;

    fn try_from(value: usize) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(SystemId::ExternalInterruptController),
            1 => Ok(SystemId::Port),
            _ => Err(SystemIdError::UnknownId),
        }
    }
}
