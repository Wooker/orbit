use crate::application_container::AppContainer;

pub trait Application<'a> {
    type Peripherals;
    const NAME: &'a str;
    fn init(&mut self);
    fn main(&mut self);
    fn interrupt(&mut self);
    fn context(&mut self) -> usize;
    fn buf(&mut self) -> usize;
    fn heap(&self) -> (usize, usize);

    #[inline(never)]
    fn to_container(&self) -> AppContainer<'a>
    where
        Self: Sized,
    {
        let struct_addr = self as *const Self as usize;
        let init_addr = Self::init as *const fn() as usize;
        let main_addr = Self::main as *const fn() as usize;
        let interrupt_addr = Self::interrupt as *const fn() as usize;
        let (h_addr, h_size) = Self::heap(&self);
        // let pmp = [PmpEntry::default(); ];
        // let peripherals = [None; ];
        AppContainer::new(
            Self::NAME,
            struct_addr,
            init_addr,
            main_addr,
            interrupt_addr,
            h_addr,
            h_size,
            // pmp,
            // peripherals,
        )
    }
    extern "C" fn ecall();
}
