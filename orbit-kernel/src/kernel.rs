#![allow(unused)]
pub mod asm;
mod message_handlers;
mod scheduler;

#[cfg(any(feature = "alloc_tlsf", feature = "alloc_llff"))]
use crate::allocator::ALLOCATOR;

use crate::{
    RINGBUF_SIZE,
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
    syscall::{InvokeLocalError, SysCall},
    systems::SystemId,
    task::{TASK_ID, Task, TaskState},
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
use orbit_arch::{
    Core, PMP,
    interface::{Interruptable, SetupMtvec},
};
use spaceport::{
    constants::{MAX_TTL, PROTOCOL_VERSION},
    error::EncodeError,
    message::Message,
    packet::{HEADER_LEN, MAX_BUFFER_LENGTH, MAX_PAYLOAD_LENGTH, Packet},
    types::Flags,
};

const DRIVERS_PER_SYSTEM: usize = 2;

pub(crate) static PACKET_ID: ID<u16> = ID::new(0);
pub static KERNEL_VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[repr(C, align(4))]
pub struct Kernel<'k> {
    context: Context,
    apps: Vec<AppContainer<'k>>,
    drivers: Vec<AppContainer<'k>>,
    systems: [Option<[usize; DRIVERS_PER_SYSTEM]>; SystemId::MAX as usize],
    interrupts: [Option<u8>; 255],
    ports: [Port<'k>; PORT_NUM],
    claims: [bool; KernelPeripherals::MAX as usize],
    core: Core<PMP>,
    clock: Clocks,
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
        let core: Core<PMP> = Core::new();

        // Enable clocks
        let mut clock = Clocks::default();
        clock.freeze();
        unsafe { init_memory() };

        // Initialize allocator
        unsafe {
            embedded_alloc::init!(
                ALLOCATOR,
                crate::chip::RAM_SIZE
                    - match usize::from_str_radix(
                        option_env!("ORBIT_KERNEL_STACK_SIZE").unwrap_or("0x1000"),
                        16
                    ) {
                        #[cfg(feature = "alloc_tlsf")]
                        Ok(v) => v + 4256,
                        #[cfg(feature = "alloc_llff")]
                        Ok(v) => v + 40,
                        Err(_) => 0,
                    }
            );
        }

        // Initialize ports
        let ports: [Port; PORT_NUM] = core::array::from_fn(|i| {
            let (ptr, interrupt, kind) = PORT_INTERRUPTS.0[i];
            core.enable_interrupt(interrupt);
            Port::new(unsafe { &*ptr }, kind)
        });

        // Save trap handler
        core.setup_mtvec(
            asm::handler as *const fn() as usize,
            crate::arch::riscv::register::mtvec::TrapMode::Direct,
        );

        let mut kernel = Self {
            context: Context::new(),
            core: Core::new(),
            ports,
            apps: Vec::new(),
            drivers: Vec::new(),
            systems: [None; SystemId::MAX as usize],
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
    pub fn version(&self) -> (&'static str) {
        (KERNEL_VERSION)
    }

    #[inline(never)]
    pub fn add_application(&mut self, app_cont: AppContainer<'k>) {
        self.apps.push(app_cont);
    }

    #[inline(never)]
    pub fn add_driver(&mut self, app_cont: AppContainer<'k>, priority: usize) {
        if let Some(mut task) = Task::new(
            Packet::new(
                Flags::empty(),
                0,
                0,
                0,
                Message::Invoke,
                app_cont.name().as_bytes(),
            ),
            priority,
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
                    if let Some(resp) = handle_invoke(
                        &mut self.apps,
                        &mut self.drivers,
                        &mut self.scheduler,
                        packet,
                    ) {
                        self.ports[i].respond(resp);
                    }
                }
                Message::KernelVersion => {
                    let (id, src, dst) = (packet.packet_id + 1, packet.dst, packet.src);
                    self.ports[i].respond(Packet::new(
                        Flags::empty(),
                        id,
                        src,
                        dst,
                        Message::Reply,
                        KERNEL_VERSION.as_bytes(),
                    ))
                }
                _ => self.ports[i].respond(Packet {
                    flags: Flags::ERROR,
                    msg_type: Message::Reply,
                    payload: b"Not implemented",
                    ..Default::default()
                }),
            }
            self.ports[i].fragments = Vec::new();
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn interrupt_handler(&mut self) {
        let code = orbit_arch::riscv::register::mcause::read().code();

        // Check if it's an external interrupt
        if let Some(driver_index) = self.interrupts.get(code).and_then(|entry| *entry) {
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
                    self.core.disable_interrupt(code);
                    self.scheduler.add(task);
                }
            }
        } else {
            self.core.disable_interrupt(code);
            self.core.enable_interrupt(code);
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
                unsafe {
                    orbit_arch::riscv::register::mepc::write(asm::wait as *const fn() as usize);
                }
                // Keep the return value from a1 before removing the completed task.
                let result = self.context.a1;

                let mut out = [0u8; MAX_BUFFER_LENGTH];
                if let Some(task) = self.scheduler.pop() {
                    if let Some(blocked) = self
                        .scheduler
                        .list_mut()
                        .iter_mut()
                        .find(|t| t.header.state == TaskState::Blocked(task.header.task_id))
                    {
                        // `a0` is the completion status and `a1` carries the
                        // successful return value.  Keeping them separate
                        // makes every `usize` value, including `usize::MAX`, a
                        // valid application result.
                        blocked.header.context.a0 = 0;
                        blocked.header.context.a1 = result;
                        blocked.header.state = TaskState::Ready;
                    }
                    // task.header
                    //     .packet
                    //     .reply(&task.header.packet.payload.len().to_le_bytes())
                    //     .encode(&mut out)
                    //     .and_then(|size| {
                    //         self.ports[0]
                    //             .send(&out[..size])
                    //             .map_err(|_| EncodeError::BufferTooSmall)
                    //     })
                    //     .unwrap_or(0);
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
                                self.core.enable_interrupt(interrupt);
                            }
                        }
                    }
                }
            }
            SysCall::RegisterSystem => {
                let task = self.scheduler.current_mut().unwrap();
                let system_id = task.header.context.a1;
                let driver_name = task.header.packet.payload;
                if let Some((index, driver)) = self
                    .drivers
                    .iter()
                    .enumerate()
                    .find(|(i, d)| d.name().as_bytes().eq(driver_name))
                {
                    if self.systems[system_id].is_none() {
                        let mut populate = [0; DRIVERS_PER_SYSTEM];
                        populate[0] = index;
                        self.systems[system_id] = Some(populate);
                        task.header.context.t0 = 0;
                    } else {
                        if let Some(system) = self.systems[system_id].as_mut() {
                            let candidate = system.iter_mut().find(|s| **s == 0);
                            if candidate.is_some() {
                                let c = candidate.unwrap();
                                *c = index;
                                task.header.context.t0 = 1;
                            } else {
                                self.drivers.remove(index);
                                self.scheduler.pop();
                            }
                        }
                    }
                }
            }
            SysCall::HandleSystem => {
                let mut scheduler = &mut self.scheduler;
                let task = scheduler.current().unwrap();
                let system_id = task.header.context.a1;
                let driver_name = task.header.packet.payload;
                if let Some((index, driver)) = self
                    .drivers
                    .iter()
                    .enumerate()
                    .find(|(i, d)| d.name().as_bytes().eq(driver_name))
                    && let Some(system_index) = self.systems[system_id]
                    && system_index.contains(&index)
                {
                    if let Ok(system_id) = SystemId::try_from(system_id) {
                        match system_id {
                            SystemId::ExternalInterruptController => {
                                let driver_index = task.header.context.t0;
                                let call_driver = self.drivers[driver_index];

                                // Create new task to handle the interrupt by driver
                                let new_task_id = TASK_ID.get_id();
                                if let Some(mut new_task) = Task::new(
                                    Packet {
                                        packet_id: new_task_id as u16,
                                        msg_type: Message::Invoke,
                                        payload: call_driver.name().as_bytes(),
                                        ..Default::default()
                                    },
                                    task.header.priority.saturating_sub(1),
                                    call_driver.interrupt_addr(),
                                ) {
                                    // Add driver struct to task context
                                    new_task.for_driver(call_driver.driver_struct().unwrap());

                                    // Mark current task as blocked by a new one
                                    scheduler.current_mut().unwrap().header.state =
                                        TaskState::Blocked(new_task_id);

                                    scheduler.current_mut().unwrap().header.context.t0 = 0;

                                    // Add new task to the scheduler
                                    scheduler.add(new_task);
                                } else {
                                    scheduler.current_mut().unwrap().header.context.t0 = 1;
                                }
                            }
                            SystemId::Port => {
                                // Create new task to handle the interrupt by driver
                                let new_task_id = TASK_ID.get_id();
                                if let Some(mut new_task) = Task::new(
                                    Packet {
                                        packet_id: new_task_id as u16,
                                        msg_type: Message::Invoke,
                                        ..Default::default()
                                    },
                                    task.header.priority.saturating_sub(1),
                                    driver.main_addr(),
                                ) {
                                    // Add driver struct to task context
                                    new_task.for_driver(driver.driver_struct().unwrap());

                                    scheduler.current_mut().unwrap().header.context.t0 = 0;

                                    // Add new task to the scheduler
                                    scheduler.add(new_task);
                                } else {
                                    scheduler.current_mut().unwrap().header.context.t0 = 6;
                                }
                            }
                            _ => {
                                scheduler.current_mut().unwrap().header.context.t0 = 7;
                            }
                        }
                    } else {
                        scheduler.current_mut().unwrap().header.context.t0 = 4;
                    }
                } else {
                    scheduler.current_mut().unwrap().header.context.t0 = 5;
                }
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
                    if let Some(system_users) =
                        self.systems[SystemId::ExternalInterruptController as usize]
                        && let Some(system_index) = system_users.get(0)
                        && let Some(system_driver) = self.drivers.get(*system_index)
                    {
                        let packet = Packet {
                            packet_id: PACKET_ID.get_id(),
                            msg_type: Message::Invoke,
                            payload: &[index.to_le_bytes(), interrupt.to_le_bytes()].concat(),
                            ..Default::default()
                        };
                        if let Some(mut task) = Task::new(packet, 1, system_driver.main_addr()) {
                            task.for_driver(system_driver.driver_struct().unwrap());
                            self.scheduler.add(task);
                        }
                    } else {
                        if interrupt < self.interrupts.len() {
                            self.interrupts[interrupt] = Some(index as u8);
                            self.core.enable_interrupt(interrupt);
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
                // Copy the syscall arguments before creating the new task.
                // The invoker's context is then free to be updated after
                // the child has been built and enqueued.
                let (name, arg, priority) = {
                    let task = self.scheduler.current().unwrap();
                    let name = unsafe {
                        core::slice::from_raw_parts(
                            task.header.context.a1 as *const u8,
                            task.header.context.a2,
                        )
                    }
                    .to_vec();
                    let arg = unsafe {
                        core::slice::from_raw_parts(
                            task.header.context.a3 as *const u8,
                            task.header.context.a4,
                        )
                    }
                    .to_vec();
                    (name, arg, task.header.priority.saturating_add(1))
                };

                let packet_id = PACKET_ID.get_id();
                let payload = [&name[..], b" ", &arg[..]].concat();
                let (mut invokee, is_application, error) = if let Some(app) =
                    self.apps.iter().find(|app| app.name().as_bytes() == name)
                {
                    (
                        Task::new(
                            Packet::new(Flags::empty(), packet_id, 0, 0, Message::Invoke, &payload),
                            priority,
                            app.main_addr(),
                        ),
                        true,
                        InvokeLocalError::TaskCreationFailed,
                    )
                } else if let Some(driver) = self
                    .drivers
                    .iter()
                    .find(|driver| driver.name().as_bytes() == name)
                {
                    let mut task = Task::new(
                        Packet::new(Flags::empty(), packet_id, 0, 0, Message::Invoke, &payload),
                        priority,
                        driver.main_addr(),
                    );
                    if let Some(task) = task.as_mut() {
                        task.for_driver(driver.driver_struct().unwrap());
                    }
                    (task, false, InvokeLocalError::TaskCreationFailed)
                } else {
                    (None, false, InvokeLocalError::ApplicationNotFound)
                };

                if let Some(mut invokee) = invokee.take() {
                    if is_application {
                        let arg_start = invokee
                            .header
                            .packet
                            .payload
                            .as_ptr()
                            .wrapping_add(name.len() + 1);
                        invokee.header.context.a0 = arg_start as usize;
                        invokee.header.context.a1 = arg.len();
                    }
                    let invokee_id = invokee.header.task_id;
                    self.scheduler.current_mut().unwrap().header.state =
                        TaskState::Blocked(invokee_id);
                    self.scheduler.add(invokee);
                    PACKET_ID.set(packet_id + 1);
                } else {
                    self.scheduler.current_mut().unwrap().header.context.a0 = error.discriminant();
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
