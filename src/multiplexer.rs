use esp_hal::{
    peripheral::{Peripheral, PeripheralRef},
    peripherals::{Peripherals, AES, SYSTEM},
};

pub enum AvailablePeripherals {
    SYSTEM,
    AES,
}

pub enum Resource {
    System(SYSTEM),
    Aes(AES),
}
impl Resource {
    fn system(system: SYSTEM) -> Self {
        Resource::System(system)
    }

    fn aes(aes: AES) -> Self {
        Resource::Aes(aes)
    }

    pub fn inner_system(self) -> Option<SYSTEM> {
        if let Resource::System(system) = self {
            Some(system)
        } else {
            None
        }
    }

    pub fn inner_aes(self) -> Option<AES> {
        if let Resource::Aes(aes) = self {
            Some(aes)
        } else {
            None
        }
    }
}

pub struct Res<T: Peripheral<P = T>> {
    inner: T,
}
impl Res<SYSTEM> {
    fn inner(self, p: impl Peripheral<P = SYSTEM>) -> SYSTEM {
        self.inner
    }
}

pub struct Multiplexer {
    peripherals: Peripherals,
}

impl Multiplexer {
    pub fn new() -> Self {
        Multiplexer {
            peripherals: Peripherals::take(),
        }
    }

    pub fn claim<'a>(&'a mut self, peripheral: AvailablePeripherals) -> Resource {
        match peripheral {
            AvailablePeripherals::SYSTEM => unsafe {
                Resource::System(self.peripherals.SYSTEM.clone_unchecked())
            },
            AvailablePeripherals::AES => unsafe {
                Resource::Aes(self.peripherals.AES.clone_unchecked())
            },
        }
    }
}
