use crate::containers::systems::SystemRecord;

pub(crate) const SYSTEM_SIZE: usize = 2;
pub trait Port {
    fn read(&mut self) -> &[u8];
    fn write(&mut self, buf: &[u8]);
}

pub trait PortAuto: Port {
    fn get_read_addr(&self) -> usize {
        Self::read as *const fn() as usize
    }

    fn get_write_addr(&self) -> usize {
        Self::write as *const fn() as usize
    }

    fn to_system_record(&self) -> SystemRecord<SYSTEM_SIZE> {
        SystemRecord {
            id: super::SystemId::Port,
            addrs: [self.get_read_addr(), self.get_write_addr()],
        }
    }
}
