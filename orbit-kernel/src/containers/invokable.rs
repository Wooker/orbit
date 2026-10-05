use super::named::NamedAuto;

#[derive(Debug, Clone, Copy)]
pub struct InvokableContainer {
    main_addr: usize,
}

impl InvokableContainer {
    pub fn new(app_main_addr: usize) -> Self {
        Self {
            main_addr: app_main_addr,
        }
    }

    pub fn main_addr(&self) -> usize {
        self.main_addr
    }
}

pub trait Invokable<'s>: NamedAuto<'s> {
    fn main(&mut self, buf: &[u8]) -> Result<usize, usize>;
}

pub trait InvokableAuto<'s>: Invokable<'s> {
    fn get_main_addr(&self) -> usize {
        Self::_main as *const fn() as usize
    }

    #[inline(never)]
    fn _main(&mut self, buf: &[u8]) {
        match self.main(buf) {
            Ok(res) => (0, res),
            Err(e) => (1, e),
        };
        unsafe { core::arch::asm!("li a0, 1;li a1, 0; ecall") };
    }

    #[inline(never)]
    fn to_invokable_container(&self) -> InvokableContainer {
        InvokableContainer::new(self.get_main_addr())
    }
}
