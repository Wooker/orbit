use crate::containers::systems::SystemRecord;

pub(crate) const SYSTEM_SIZE: usize = 3;
pub trait ExternalInterruptController {
    fn enable_interrupt(&mut self, irq: usize);
    fn disable_interrupt(&mut self);
    fn dispatch(&mut self) -> usize;
}

pub trait ExternalInterruptControllerAuto: ExternalInterruptController {
    fn get_enable_addr(&self) -> usize {
        Self::enable_interrupt as *const fn() as usize
    }

    fn get_disable_addr(&self) -> usize {
        Self::disable_interrupt as *const fn() as usize
    }

    fn get_dispatch_addr(&self) -> usize {
        Self::dispatch as *const fn() as usize
    }

    fn to_system_record(&self) -> SystemRecord<SYSTEM_SIZE> {
        SystemRecord {
            id: super::SystemId::ExternalInterruptController,
            addrs: [
                self.get_enable_addr(),
                self.get_disable_addr(),
                self.get_dispatch_addr(),
            ],
        }
    }
}
