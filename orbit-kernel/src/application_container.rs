#[derive(Clone, Copy)]
pub struct AppContainer<'a> {
    name: &'a str,
    driver_struct: Option<usize>,
    init_addr: usize,
    main_addr: usize,
    interrupt_addr: usize,
    // pmp: [PmpEntry; PMP_REGS],
    // peripherals: [Option<KernelPeripherals>; PMP_REGS],
}

impl<'a> AppContainer<'a> {
    pub fn new(
        name: &'a str,
        app_struct: Option<usize>,
        app_init_addr: usize,
        app_main_addr: usize,
        app_interrupt_addr: usize,
        // pmp: [PmpEntry; PMP_REGS],
        // peripherals: [Option<KernelPeripherals>; PMP_REGS],
    ) -> Self {
        // if PMP_REGS > 0 {
        //     let mut pmps: [PmpEntry; PMP_REGS] = [PmpEntry::default(); PMP_REGS];
        //     pmps[0].address = unsafe { &_app_uart_text_main as *const usize as usize };
        // }
        Self {
            name,
            driver_struct: app_struct,
            init_addr: app_init_addr,
            main_addr: app_main_addr,
            interrupt_addr: app_interrupt_addr,
            // pmp,
            // peripherals,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn driver_struct(&self) -> &Option<usize> {
        &self.driver_struct
    }

    pub fn init_addr(&self) -> usize {
        self.init_addr
    }

    pub fn main_addr(&self) -> usize {
        self.main_addr
    }

    pub fn interrupt_addr(&self) -> usize {
        self.interrupt_addr
    }

    // pub fn get_pmp(&self) -> [PmpEntry; PMP_REGS] {
    //     self.pmp
    // }
}

#[repr(usize)]
#[derive(Clone, PartialEq, Eq)]
pub enum RunApplication {
    None,
    Init,
    Main,
    Interrupt,
    Jumped,
    Abort,
}
