use crate::containers::systems::SystemRecord;

pub(crate) const SYSTEM_SIZE: usize = 3;
pub trait ExternalInterruptController {
    fn enable_interrupt(&mut self, index: usize, irq: usize);
    fn disable_interrupt(&mut self, irq: usize);
    fn dispatch(&mut self, irq: usize) -> usize;
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
    fn _enable_interrupt(&mut self, index: usize, irq: usize) {
        self.enable_interrupt(index, irq);
        unsafe { core::arch::asm!("li a0, 1;li a1, 0; ecall") };
    }
    fn _disable_interrupt(&mut self, irq: usize) {
        self.disable_interrupt(irq);
        unsafe { core::arch::asm!("li a0, 1;li a1, 0; ecall") };
    }
    fn _dispatch(&mut self, irq: usize) {
        let _index = self.dispatch(irq);
        unsafe { core::arch::asm!("li a0, 1;li a1, 0; ecall") };
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
