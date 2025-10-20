use core::ops::Add;

use crate::{
    // claim::KernelPeripherals,
    context::Context,
    // pmp_entry::PmpEntry,
    ringbuf::RingBuf,
    {RINGBUF_SIZE, RingbufType},
};

#[derive(Clone, Copy)]
pub struct AppContainer<'a> {
    name: &'a str,
    struct_addr: usize,
    init_addr: usize,
    main_addr: usize,
    interrupt_addr: usize,
    // pmp: [PmpEntry; PMP_REGS],
    // peripherals: [Option<KernelPeripherals>; PMP_REGS],
}

impl<'a> AppContainer<'a> {
    pub fn new(
        name: &'a str,
        app_struct: usize,
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
            struct_addr: app_struct,
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

    pub fn struct_addr(&self) -> usize {
        self.struct_addr
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

    pub fn context(&self) -> &mut Context {
        unsafe { &mut *(self.struct_addr as *mut Context) }
    }

    pub fn buf(&mut self) -> &mut RingBuf<RINGBUF_SIZE, RingbufType> {
        unsafe {
            &mut *(self.struct_addr.add(core::mem::size_of::<Context>())
                as *mut RingBuf<RINGBUF_SIZE, RingbufType>)
        }
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
