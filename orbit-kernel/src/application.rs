use crate::application_container::AppContainer;

pub trait Application<'a> {
    type Peripherals;
    const NAME: &'a str;
    fn init(&mut self);
    fn main(&mut self, buf: &[u8]);
    fn interrupt(&mut self);

    #[inline(never)]
    fn to_container(&self) -> AppContainer<'a>
    where
        Self: Sized,
    {
        let init_addr = Self::init as *const fn() as usize;
        let main_addr = Self::main as *const fn() as usize;
        let interrupt_addr = Self::interrupt as *const fn() as usize;
        // let pmp = [PmpEntry::default(); ];
        // let peripherals = [None; ];
        AppContainer::new(
            Self::NAME,
            None,
            init_addr,
            main_addr,
            interrupt_addr,
            // pmp,
            // peripherals,
        )
    }
    extern "C" fn ecall();
}
