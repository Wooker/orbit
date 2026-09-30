pub mod external_interrupt_controller;
pub mod port;

#[repr(u8)]
#[derive(Debug, Clone, Copy)]
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

#[allow(dead_code)]
#[derive(Debug, Clone, Copy)]
pub struct SystemRecord<const N: usize> {
    pub(crate) id: SystemId,
    pub(crate) addrs: [usize; N],
}

#[derive(Debug, Default, Copy, Clone)]
pub struct SystemContainer {
    pub external_interrupt_controller:
        Option<SystemRecord<{ external_interrupt_controller::SYSTEM_SIZE }>>,
    pub port: Option<SystemRecord<{ port::SYSTEM_SIZE }>>,
}
