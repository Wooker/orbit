#![allow(unused)]

use core::{mem::ManuallyDrop, ops::Add};

use crate::{
    claim::KernelPeripherals,
    port::{ringbuf::RingBuf, RingbufType, RINGBUF_SIZE},
};
use orbit_arch::{riscv::register::Permission, riscv::register::Range, PMP};

#[derive(Clone, Copy)]
pub struct PmpEntry {
    pub address: usize,
    pub range: Range,
    pub permission: Permission,
    pub locked: bool,
}

// NAPOT: addr >> 2 | ((1<<(pow-3)-1)
// OTHER: addr >> 2
impl PmpEntry {
    #[allow(unused)]
    pub fn new(addr: usize, range: Range, permission: Permission, locked: bool) -> Self {
        Self {
            address: addr,
            range,
            permission,
            locked,
        }
    }
}
impl Default for PmpEntry {
    fn default() -> Self {
        Self {
            address: 0x0,
            range: Range::OFF,
            permission: Permission::NONE,
            locked: false,
        }
    }
}

#[repr(usize)]
#[derive(Clone, PartialEq, Eq)]
pub enum RunApplication {
    None,
    Main,
    Interrupt,
    Jumped,
    Abort,
}

pub trait Application<'a> {
    fn init(&mut self);
    fn main(&mut self);
    fn interrupt(&mut self);
    fn stack_size() -> usize;
    fn context(&mut self) -> usize;
    fn buf(&mut self) -> usize;
    #[inline(never)]
    fn to_container(&mut self, name: &'a str, struct_addr: usize) -> AppContainer<'a, PMP>
    where
        Self: Sized,
    {
        let main_addr = Self::main as *const fn() as usize;
        let interrupt_addr = Self::interrupt as *const fn() as usize;
        // ManuallyDrop::new(self);
        AppContainer {
            name,
            struct_addr,
            main_addr,
            interrupt_addr,
            pmp: [PmpEntry::default(); PMP],
            peripherals: [None; PMP],
        }
    }
    extern "C" fn ecall();
}

#[derive(Clone, Copy)]
pub struct AppContainer<'a, const PMP_REGS: usize> {
    name: &'a str,
    struct_addr: usize,
    main_addr: usize,
    interrupt_addr: usize,
    pmp: [PmpEntry; PMP_REGS],
    peripherals: [Option<KernelPeripherals>; PMP_REGS],
}

/* CONTEXT */
#[derive(Clone, Copy)]
pub struct Context {
    pub ra: usize,
    pub sp: usize,
    pub gp: usize,
    pub tp: usize,
    pub t0: usize,
    pub t1: usize,
    pub t2: usize,
    pub s0: usize,
    pub s1: usize,
    pub a0: usize,
    pub a1: usize,
    pub a2: usize,
    pub a3: usize,
    pub a4: usize,
    pub a5: usize,
    #[cfg(not(target_feature = "e"))]
    pub a6: usize,
    #[cfg(not(target_feature = "e"))]
    pub a7: usize,
    #[cfg(not(target_feature = "e"))]
    pub s2: usize,
    #[cfg(not(target_feature = "e"))]
    pub s3: usize,
    #[cfg(not(target_feature = "e"))]
    pub s4: usize,
    #[cfg(not(target_feature = "e"))]
    pub s5: usize,
    #[cfg(not(target_feature = "e"))]
    pub s6: usize,
    #[cfg(not(target_feature = "e"))]
    pub s7: usize,
    #[cfg(not(target_feature = "e"))]
    pub s8: usize,
    #[cfg(not(target_feature = "e"))]
    pub s9: usize,
    #[cfg(not(target_feature = "e"))]
    pub s10: usize,
    #[cfg(not(target_feature = "e"))]
    pub s11: usize,
    #[cfg(not(target_feature = "e"))]
    pub t3: usize,
    #[cfg(not(target_feature = "e"))]
    pub t4: usize,
    #[cfg(not(target_feature = "e"))]
    pub t5: usize,
    #[cfg(not(target_feature = "e"))]
    pub t6: usize,
    #[cfg(not(target_feature = "e"))]
    pub mepc: usize,
}

impl Context {
    pub const fn new() -> Self {
        Self {
            ra: 0,
            sp: 0,
            gp: 0,
            tp: 0,
            t0: 0,
            t1: 0,
            t2: 0,
            s0: 0,
            s1: 0,
            a0: 0,
            a1: 0,
            a2: 0,
            a3: 0,
            a4: 0,
            a5: 0,
            #[cfg(not(target_feature = "e"))]
            a6: 0,
            #[cfg(not(target_feature = "e"))]
            a7: 0,
            #[cfg(not(target_feature = "e"))]
            s2: 0,
            #[cfg(not(target_feature = "e"))]
            s3: 0,
            #[cfg(not(target_feature = "e"))]
            s4: 0,
            #[cfg(not(target_feature = "e"))]
            s5: 0,
            #[cfg(not(target_feature = "e"))]
            s6: 0,
            #[cfg(not(target_feature = "e"))]
            s7: 0,
            #[cfg(not(target_feature = "e"))]
            s8: 0,
            #[cfg(not(target_feature = "e"))]
            s9: 0,
            #[cfg(not(target_feature = "e"))]
            s10: 0,
            #[cfg(not(target_feature = "e"))]
            s11: 0,
            #[cfg(not(target_feature = "e"))]
            t3: 0,
            #[cfg(not(target_feature = "e"))]
            t4: 0,
            #[cfg(not(target_feature = "e"))]
            t5: 0,
            #[cfg(not(target_feature = "e"))]
            t6: 0,
            #[cfg(not(target_feature = "e"))]
            mepc: 0,
        }
    }
}

impl<'a, const PMP_REGS: usize> AppContainer<'a, PMP_REGS> {
    pub fn new(
        name: &'a str,
        app_struct: usize,
        app_main_addr: usize,
        app_interrupt_addr: usize,
        pmp: [PmpEntry; PMP_REGS],
        peripherals: [Option<KernelPeripherals>; PMP_REGS],
    ) -> Self {
        // if PMP_REGS > 0 {
        //     let mut pmps: [PmpEntry; PMP_REGS] = [PmpEntry::default(); PMP_REGS];
        //     pmps[0].address = unsafe { &_app_uart_text_main as *const usize as usize };
        // }
        Self {
            name,
            struct_addr: app_struct,
            main_addr: app_main_addr,
            interrupt_addr: app_interrupt_addr,
            pmp,
            peripherals,
        }
    }

    pub fn name(&self) -> &str {
        self.name
    }

    pub fn struct_addr(&self) -> usize {
        self.struct_addr
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

    pub fn get_pmp(&self) -> [PmpEntry; PMP_REGS] {
        self.pmp
    }
}
