use crate::{
    containers::systems::{SystemId, SystemRecord},
    syscall::SysCall,
};

pub(crate) const SYSTEM_SIZE: usize = 2;
pub trait Port {
    fn read(&mut self) -> (Option<usize>, &[u8]);
    fn write(&mut self, buf: &[u8]);
}

pub trait PortAuto: Port {
    fn get_read_addr(&self) -> usize {
        Self::_read as *const fn() as usize
    }

    fn get_write_addr(&self) -> usize {
        Self::_write as *const fn() as usize
    }

    fn _read(&mut self) -> ! {
        let (opt_task_id, slice) = self.read();
        let syscall = SysCall::ReturnSystem as usize;
        let system_id = SystemId::Port as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") system_id,
                in("a2") opt_task_id.unwrap_or(usize::MAX),
                in("a3") if slice.len() == 0 { 0 as *const u8 } else { slice.as_ptr() },
                in("a4") slice.len(),
                options(noreturn)
            )
        }
    }
    fn _write(&mut self, buf: &[u8]) -> ! {
        self.write(buf);
        let syscall = SysCall::ReturnSystem as usize;
        let system_id = SystemId::Port as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") system_id,
                in("a2") 0,
                in("a3") 0,
                options(noreturn)
            )
        };
    }

    fn to_system_record(&self) -> SystemRecord<SYSTEM_SIZE> {
        SystemRecord {
            id: super::SystemId::Port,
            addrs: [self.get_read_addr(), self.get_write_addr()],
        }
    }
}
