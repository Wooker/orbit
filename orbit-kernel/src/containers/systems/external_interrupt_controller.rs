use crate::{
    containers::systems::{SystemId, SystemRecord},
    syscall::SysCall,
};

pub(crate) const SYSTEM_SIZE: usize = 3;
pub trait ExternalInterruptController {
    fn enable_interrupt(&mut self, index: usize, irq: usize);
    fn disable_interrupt(&mut self, irq: usize);
    fn dispatch(&mut self) -> usize;
}

pub trait ExternalInterruptControllerAuto: ExternalInterruptController {
    fn get_enable_addr(&self) -> usize {
        Self::_enable_interrupt as *const fn() as usize
    }

    fn get_disable_addr(&self) -> usize {
        Self::_disable_interrupt as *const fn() as usize
    }

    fn get_dispatch_addr(&self) -> usize {
        Self::_dispatch as *const fn() as usize
    }

    // TODO: Handle system returns properly by kernel
    // Now they return as from main
    fn _enable_interrupt(&mut self, index: usize, irq: usize) -> ! {
        self.enable_interrupt(index, irq);
        let syscall = SysCall::ReturnSystem as usize;
        let system_id = SystemId::ExternalInterruptController as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") system_id,
                in("a2") 0,
                options(noreturn)
            )
        };
    }
    fn _disable_interrupt(&mut self, irq: usize) -> ! {
        self.disable_interrupt(irq);
        let syscall = SysCall::ReturnSystem as usize;
        let system_id = SystemId::ExternalInterruptController as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") system_id,
                in("a2") usize::MAX,
                options(noreturn)
            )
        };
    }
    fn _dispatch(&mut self) -> ! {
        let index = self.dispatch();
        let syscall = SysCall::ReturnSystem as usize;
        let system_id = SystemId::ExternalInterruptController as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") system_id,
                in("a2") index,
                options(noreturn)
            )
        };
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
