#![allow(unused)]
pub mod asm;
mod message_handlers;
mod scheduler;

use crate::{
    RINGBUF_SIZE,
    allocator::{ALLOCATOR, SimpleAllocator},
    application::Application,
    application_container::{AppContainer, RunApplication},
    claim::KernelPeripherals,
    clock::Clocks,
    context::Context,
    id::ID,
    kernel::{message_handlers::handle_invoke, scheduler::Scheduler},
    port::{
        Port,
        port_kind::{PORT_INTERRUPTS, PORT_NUM},
    },
    ringbuf::RingBuf,
    syscall::SysCall,
    task::{Task, TaskState},
};
use alloc::{boxed::Box, collections::linked_list::LinkedList, slice, vec::Vec};
use core::{
    arch::{asm, naked_asm},
    cell::UnsafeCell,
    mem::{MaybeUninit, transmute},
    panic::PanicInfo,
    pin::Pin,
    ptr::null,
};
use orbit_arch::{Core, PMP};
use spaceport::{
    constants::{MAX_TTL, PROTOCOL_VERSION},
    error::EncodeError,
    message::Message,
    packet::{HEADER_LEN, MAX_BUFFER_LENGTH, MAX_PAYLOAD_LENGTH, Packet},
    types::Flags,
};

pub const APPS: usize = 5;
pub(crate) static PACKET_ID: ID<u16> = ID::new(0);
pub static KERNEL_MAJOR: u8 = 0;
pub static KERNEL_MINOR: u8 = 1;

#[repr(C, align(4))]
pub struct Kernel<'k> {
    context: Context,
    apps: Vec<AppContainer<'k>>,
    drivers: Vec<AppContainer<'k>>,
    interrupts: [Option<u8>; 255],
    ports: [Port<'k>; PORT_NUM],
    claims: [bool; KernelPeripherals::MAX as usize],
    pub core: Core<PMP>,
    pub clock: Clocks,
    scheduler: Scheduler<'k>,
}

unsafe extern "C" {
    static mut _sidata: u32;
    static mut _sdata: u32;
    static mut _edata: u32;
    static mut _sbss: u32;
    static mut _ebss: u32;
}

fn init_memory() {
    unsafe {
        // Copy .data
        let mut src = &raw const _sidata as *const u32;
        let mut dst = &raw mut _sdata as *mut u32;

        while dst < &raw mut _edata {
            *dst = *src;
            dst = dst.add(1);
            src = src.add(1);
        }

        // Zero .bss
        let mut bss = &raw mut _sbss as *mut u32;
        while bss < &raw mut _ebss {
            *bss = 0;
            bss = bss.add(1);
        }
    }
}

impl<'k> Kernel<'k> {
    #[rustc_align(4)]
    #[inline(never)]
    pub fn new() -> Self {
        // Enable clocks
        let mut clock = Clocks::default();
        clock.freeze();
        unsafe { init_memory() };

        // Initialize ports
        let ports: [Port; PORT_NUM] = core::array::from_fn(|i| {
            let (ptr, interrupt, kind) = PORT_INTERRUPTS.0[i];
            unsafe {
                orbit_arch::pfic::enable_interrupt(interrupt as u8);
            }
            Port::new(unsafe { &*ptr }, kind)
        });

        // Save trap handler
        unsafe {
            crate::arch::riscv::register::mtvec::write(
                asm::handler as *const fn() as usize,
                crate::arch::riscv::register::mtvec::TrapMode::Direct,
            )
        };

        let mut kernel = Self {
            context: Context::new(),
            core: Core::new(),
            ports,
            apps: Vec::new(),
            drivers: Vec::new(),
            interrupts: [None; 255],
            claims: [false; KernelPeripherals::MAX as usize],
            clock,
            scheduler: Scheduler::new(),
        };

        unsafe {
            asm!("mv {0}, sp", out(reg) kernel.context.sp);
        }

        kernel
    }

    #[inline(never)]
    #[unsafe(link_section = ".text")]
    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }

    #[inline(never)]
    pub fn add_application(&mut self, app_cont: AppContainer<'k>) {
        self.apps.push(app_cont);
    }

    #[inline(never)]
    pub fn add_driver(&mut self, app_cont: AppContainer<'k>) {
        if let Some(mut task) = Task::new(
            Packet::new(
                Flags::empty(),
                0,
                0,
                0,
                Message::Invoke,
                app_cont.name().as_bytes(),
            ),
            1,
            app_cont.init_addr(),
        ) {
            task.for_driver(app_cont.driver_struct().unwrap());
            self.drivers.push(app_cont);
            self.scheduler.add(task);
        }
    }

    // #[inline(never)]
    // fn set_pmp(&mut self, app: &AppContainer<PMP>) {
    //     for (i, pe) in app.get_pmp().iter().enumerate() {
    //         let _ = self
    //             .core
    //             .pmp
    //             .write_cfg(0, i, pe.range, pe.permission, pe.locked);
    //         let _ = self.core.pmp.write_addr(i, pe.address);
    //     }
    // }

    #[inline(never)]
    pub fn clock(&self) -> usize {
        self.clock.hclk
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn port_handler(&mut self, i: usize) {
        if let Some(packet) = self.ports[i].handle() {
            match packet.msg_type {
                Message::Invoke => {
                    let resp = handle_invoke(
                        &mut self.apps,
                        &mut self.drivers,
                        &mut self.scheduler,
                        packet,
                    );
                    self.ports[i].respond(resp);
                }
                Message::KernelVersion => {
                    let (id, src, dst) = (packet.packet_id + 1, packet.dst, packet.src);
                    self.ports[i].respond(Packet::new(
                        Flags::empty(),
                        id,
                        src,
                        dst,
                        Message::Reply,
                        &[KERNEL_MAJOR, KERNEL_MINOR],
                    ))
                }
                _ => self.ports[i].respond(Packet {
                    version: PROTOCOL_VERSION,
                    flags: Flags::ERROR,
                    packet_id: 0,
                    src: 0,
                    dst: 0,
                    ttl: 0,
                    msg_type: Message::Reply,
                    payload: b"Not implemented",
                }),
            }
            self.ports[i].fragments = Vec::new();
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn interrupt_handler(&mut self) {
        let code = orbit_arch::riscv::register::mcause::read().code();

        // Check if it's a port interrupt
        if let Some((index, port)) = PORT_INTERRUPTS
            .0
            .iter()
            .enumerate()
            .find(|(_, (_, interrupt, _))| *interrupt == code)
        {
            self.port_handler(index);
        } else if let Some(driver_index) = self.interrupts.get(code).and_then(|entry| *entry) {
            // Spawn the registered driver's interrupt routine.
            if let Some(driver) = self.drivers.get(driver_index as usize) {
                let packet = Packet::new(
                    Flags::empty(),
                    PACKET_ID.get_id(),
                    0,
                    0,
                    Message::Invoke,
                    driver.name().as_bytes(),
                );
                if let Some(mut task) = Task::new(packet, 1, driver.interrupt_addr()) {
                    task.for_driver(driver.driver_struct().unwrap());
                    unsafe { orbit_arch::pfic::disable_interrupt(code as u8) };
                    self.scheduler.add(task);
                }
            }
        } else {
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn syscall_handler(&mut self) {
        let self_addr = self as *const Kernel as usize;
        let syscall = {
            let task = self.scheduler.current_mut().unwrap();

            task.header.context.mepc = orbit_arch::riscv::register::mepc::read() + 4;
            let task_addr = &**task as *const Task as *const () as usize;
            SysCall::from_usize(task.header.context.a0)
        };

        match syscall {
            SysCall::ReturnMain => {
                orbit_arch::riscv::register::mepc::write(asm::wait as *const fn() as usize);
                self.context.a1 = 0;

                let mut out = [0u8; MAX_BUFFER_LENGTH];
                if let Some(task) = self.scheduler.pop() {
                    if let Some(blocked) = self
                        .scheduler
                        .list_mut()
                        .iter_mut()
                        .find(|t| t.header.state == TaskState::Blocked(task.header.task_id))
                    {
                        blocked.header.context.a0 = task.header.context.a1;
                        blocked.header.state = TaskState::Ready;
                    }
                    task.header
                        .packet
                        .reply(&task.header.packet.payload.len().to_le_bytes())
                        .encode(&mut out)
                        .and_then(|size| {
                            self.ports[0]
                                .send(&out[..size])
                                .map_err(|_| EncodeError::BufferTooSmall)
                        })
                        .unwrap_or(0);
                }
            }
            SysCall::ReturnInit => if let Some(task) = self.scheduler.pop() {},
            SysCall::ReturnInterrupt => {
                if let Some(task) = self.scheduler.pop() {
                    if let Some((driver_index, _)) = self
                        .drivers
                        .iter()
                        .enumerate()
                        .find(|(i, d)| d.name().as_bytes() == task.header.packet.payload)
                    {
                        for (interrupt, i) in self.interrupts.iter().enumerate() {
                            if let Some(index) = i
                                && *index == driver_index as u8
                            {
                                unsafe { orbit_arch::pfic::enable_interrupt(interrupt as u8) };
                            }
                        }
                    }
                }
                // unsafe { orbit_arch::pfic::enable_interrupt(40) };
            }
            SysCall::RegisterInterrupt => {
                let task = self.scheduler.current().unwrap();
                let interrupt = task.header.context.a1;
                let driver_name = task.header.packet.payload;
                if let Some((index, driver)) = self
                    .drivers
                    .iter()
                    .enumerate()
                    .find(|(i, d)| d.name().as_bytes().eq(driver_name))
                {
                    if interrupt < self.interrupts.len() {
                        self.interrupts[interrupt] = Some(index as u8);
                        unsafe {
                            orbit_arch::pfic::enable_interrupt(interrupt as u8);
                        }
                    }
                }
            }
            SysCall::Send => {
                let task = self.scheduler.current_mut().unwrap();
                let buf = unsafe {
                    core::slice::from_raw_parts(
                        task.header.context.a1 as *const u8,
                        task.header.context.a2,
                    )
                };
                let mut out = [0u8; MAX_BUFFER_LENGTH];
                task.header
                    .packet
                    .reply(buf)
                    .encode(&mut out)
                    .and_then(|size| {
                        self.ports[0]
                            .send(&out[..size])
                            .map_err(|_| EncodeError::BufferTooSmall)
                    })
                    .unwrap_or(0);
            }
            SysCall::InvokeLocal => {
                let task = self.scheduler.current_mut().unwrap();
                let name = unsafe {
                    core::slice::from_raw_parts(
                        task.header.context.a1 as *const u8,
                        task.header.context.a2,
                    )
                };
                let arg = unsafe {
                    core::slice::from_raw_parts(
                        task.header.context.a3 as *const u8,
                        task.header.context.a4,
                    )
                };
                if let Some(app) = self.apps.iter().find(|a| a.name().as_bytes() == name) {
                    let packet_id = PACKET_ID.get_id();
                    PACKET_ID.set(packet_id + 1);
                    if let Some(t) = Task::new(
                        Packet::new(
                            Flags::empty(),
                            packet_id,
                            0,
                            0,
                            Message::Invoke,
                            &[name, b" ", arg].concat(),
                        ),
                        task.header.priority.saturating_add(1),
                        app.main_addr(),
                    ) {
                        task.header.state = TaskState::Blocked(t.header.task_id);
                    }
                } else if let Some(driver) =
                    self.drivers.iter().find(|d| d.name().as_bytes() == name)
                {
                    let packet_id = PACKET_ID.get_id();
                    PACKET_ID.set(packet_id + 1);
                    if let Some(mut t) = Task::new(
                        Packet::new(
                            Flags::empty(),
                            packet_id,
                            0,
                            0,
                            Message::Invoke,
                            &[name, b" ", arg].concat(),
                        ),
                        task.header.priority.saturating_add(1),
                        driver.main_addr(),
                    ) {
                        task.header.state = TaskState::Blocked(t.header.task_id);
                        t.for_driver(driver.driver_struct().unwrap());
                    }
                } else {
                    task.header.context.a0 = usize::MAX;
                }
            }
            SysCall::Invoke => {
                let task = self.scheduler.current_mut().unwrap();
                let name = unsafe {
                    core::slice::from_raw_parts(
                        task.header.context.a1 as *const u8,
                        task.header.context.a2,
                    )
                };
                let arg = unsafe {
                    core::slice::from_raw_parts(
                        task.header.context.a3 as *const u8,
                        task.header.context.a4,
                    )
                };
                let payload = &[name, b" ", arg].concat();
                let packet_id = PACKET_ID.get_id();
                PACKET_ID.set(packet_id + 1);
                let packet = Packet::new(Flags::empty(), packet_id, 0, 0, Message::Invoke, payload);
                let mut arr = [0; MAX_BUFFER_LENGTH];
                if let Ok(size) = packet.encode(&mut arr) {
                    self.ports.iter_mut().for_each(|p| {
                        p.send(&arr);
                    });
                    task.header.context.a0 = size;
                } else {
                    task.header.context.a0 = 0;
                }
            }
            _ => {}
        };
    }

    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn setup_event_loop(&mut self) -> ! {
        if let Some(task) = self.scheduler.current_mut() {
            let task_addr = task.as_ref().get_ref() as *const Task as *const ();
            let addr = task.header.context.mepc;
            task.header.state = TaskState::Ready;
            // self.set_pmp(&app_cont);
            unsafe { asm::context_switch(self as *const Kernel as usize, task_addr as usize, addr) }
        } else {
            unsafe { asm::exit_to_loop() }
        }
    }
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn handle_panic(&mut self, panic_info: &PanicInfo) {
        // for port in self.ports.iter_mut() {
        //     let msg = panic_info.message().as_str().unwrap();
        //     for chunk in msg.as_bytes().chunks(32) {
        //         port.write(chunk);
        //     }
        // }
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub unsafe extern "C" fn initialize_finish() -> ! {
        naked_asm!(
            "
            la t0, wait;
            csrw mepc, t0;
            mret;
            ",
        );
    }
}
