use crate::syscall::SysCall;

#[derive(Debug, Clone, Copy)]
pub struct DriverContainer {
    driver_init_addr: usize,
    driver_interrupt_addr: usize,
}
impl<'d> DriverContainer {
    pub fn new(driver_init_addr: usize, driver_interrupt_addr: usize) -> Self {
        Self {
            driver_init_addr,
            driver_interrupt_addr,
        }
    }

    pub fn init_addr(&self) -> usize {
        self.driver_init_addr
    }

    pub fn interrupt_addr(&self) -> usize {
        self.driver_interrupt_addr
    }
}

pub trait Driver<'d> {
    fn init(&mut self);
    fn interrupt(&mut self);
}

pub trait DriverAuto<'d>: Driver<'d> {
    type Peripherals;

    fn get_init_addr(&self) -> usize {
        Self::_init as *const fn() as usize
    }
    fn get_interrupt_addr(&self) -> usize {
        Self::_interrupt as *const fn() as usize
    }

    #[inline(never)]
    fn _init(&mut self) -> ! {
        self.init();
        let syscall = SysCall::ReturnInit as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") 0,
                options(noreturn)
            )
        };
    }

    #[inline(never)]
    fn _interrupt(&mut self) -> ! {
        self.interrupt();
        let syscall = SysCall::ReturnInterrupt as usize;
        unsafe {
            core::arch::asm!(
                "ecall",
                in("a0") syscall,
                in("a1") 0,
                options(noreturn)
            )
        };
    }

    #[inline(never)]
    fn to_driver_container(&self) -> DriverContainer
    where
        Self: Sized + Driver<'d>,
    {
        DriverContainer::new(self.get_init_addr(), self.get_interrupt_addr())
    }
}
