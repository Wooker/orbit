#![allow(unused)]
pub mod asm;
mod scheduler;
pub mod systems;

#[cfg(any(feature = "alloc_tlsf", feature = "alloc_llff"))]
use crate::allocator::ALLOCATOR;

use crate::{
    RINGBUF_SIZE,
    application::Application,
    application_container::{AppContainer, RunApplication},
    claim::KernelPeripherals,
    containers::{Container, systems::SystemId},
    context::Context,
    id::ID,
    kernel::{scheduler::Scheduler, systems::Systems},
    ringbuf::RingBuf,
    syscall::{InvokeLocalError, SysCall},
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
    packet::{HEADER_LEN, MAX_BUFFER_LENGTH, MAX_PAYLOAD_LENGTH},
    types::Flags,
};

const CONTAINERS: usize =
    usize::from_str_radix(option_env!("ORBIT_KERNEL_CONTAINERS").unwrap_or("0"), 10).unwrap_or(0);

pub(crate) static PACKET_ID: ID<u16> = ID::new(0);

#[macro_export]
macro_rules! kernel_version {
    () => {
        "0.1.2"
    };
}

#[repr(C, align(4))]
pub struct Kernel<'k> {
    context: Context,
    containers: [Option<Container<'k>>; CONTAINERS],
    systems: systems::Systems,
    interrupts: [Option<u8>; 255],
    claims: [bool; KernelPeripherals::MAX as usize],
    core: Core<PMP>,
    scheduler: Scheduler,
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
        // Save trap handler
        core.setup_mtvec(
            asm::handler as *const fn() as usize,
            crate::arch::riscv::register::mtvec::TrapMode::Direct,
        );

        let mut kernel = Self {
            context: Context::new(),
            core: Core::new(),
            containers: [None; CONTAINERS],
            systems: Systems::default(),
            interrupts: [None; 255],
            claims: [false; KernelPeripherals::MAX as usize],
            scheduler: Scheduler::new(),
        };

        unsafe {
            asm!("mv {0}, sp", out(reg) kernel.context.sp);
        }

        kernel
    }

    // #[inline(never)]
    // #[unsafe(link_section = ".text")]
    // pub fn version(&self) -> (&'static str) {
    //     (KERNEL_VERSION)
    // }

    #[inline(never)]
    pub fn add_container(&mut self, container: Container<'k>) {
        if let Some((index, c)) = self
            .containers
            .iter_mut()
            .enumerate()
            .find(|c| c.1.is_none())
        {
            if let Some(named) = container.named
                && let Some(data) = container.data
                && let Some(driver) = container.driver
                && let name = named.name()
            {
                if let Some(mut task) = Task::new(name.as_bytes(), index, driver.init_addr()) {
                    task.for_driver(data.data());
                    self.scheduler.add(task);
                }
                *c = Some(container);
            }
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
    #[unsafe(no_mangle)]
    fn interrupt_handler(&mut self) {
        let code = orbit_arch::riscv::register::mcause::read().code();

        // Check if interrupt is registered
        if let Some(container_index) = self.interrupts.get(code).and_then(|entry| *entry) {
            // Spawn the registered driver's interrupt routine.
            if let Some(container) = self.containers.get(container_index as usize).unwrap()
                && let Some(named) = container.named
                && let Some(driver) = container.driver
                && let name = named.name()
                && let Some(data) = container.data
            {
                if let Some(mut task) = Task::new(name.as_bytes(), 1, driver.interrupt_addr()) {
                    task.for_driver(data.data());
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
                    //     .reply(&task.payload.len().to_le_bytes())
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
                    if let Some((driver_index, _)) =
                        self.containers.iter().enumerate().find(|(i, d)| {
                            if let Some(c) = d
                                && let Some(named) = c.named
                                && let name = named.name()
                                && task.payload.get(..name.len()) == Some(name.as_bytes())
                            {
                                true
                            } else {
                                false
                            }
                        })
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
            SysCall::RegisterSystem => self.handle_register_system(),
            SysCall::HandleSystem | SysCall::ReturnSystem => self.handle_system(),
            SysCall::RegisterInterrupt => {
                self.handle_register_interrupt();
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
                // TODO: handle via Port system
                // task.header
                //     .packet
                //     .reply(buf)
                //     .encode(&mut out)
                //     .and_then(|size| {
                //         self.ports[0]
                //             .send(&out[..size])
                //             .map_err(|_| EncodeError::BufferTooSmall)
                //     })
                //     .unwrap_or(0);
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
                    };
                    let arg = unsafe {
                        core::slice::from_raw_parts(
                            task.header.context.a3 as *const u8,
                            task.header.context.a4,
                        )
                    };
                    (name, arg, task.header.priority.saturating_add(1))
                };

                let packet_id = PACKET_ID.get_id();
                let (mut invokee, is_application, error) =
                    if let Some(app) = self.containers.iter().find(|app| {
                        app.is_some()
                            && app.unwrap().named.is_some()
                            && app.unwrap().named.unwrap().name().as_bytes().eq(name)
                    }) && let Some(container) = app
                        && let Some(invokable) = container.invokable
                    {
                        (
                            Task::new(
                                &[&name[..], b" ", &arg[..]].concat(),
                                priority,
                                invokable.main_addr(),
                            ),
                            true,
                            InvokeLocalError::TaskCreationFailed,
                        )
                    } else if let Some(driver) = self.containers.iter().find(|app| {
                        app.is_some()
                            && app.unwrap().named.is_some()
                            && app.unwrap().named.unwrap().name().as_bytes().eq(name)
                    }) && let Some(container) = driver
                        && let Some(invokable) = container.invokable
                        && let Some(data) = container.data
                    {
                        let mut task = Task::new(
                            &[&name[..], b" ", &arg[..]].concat(),
                            priority,
                            invokable.main_addr(),
                        );
                        if let Some(task) = task.as_mut() {
                            task.for_driver(data.data());
                        }
                        (task, false, InvokeLocalError::TaskCreationFailed)
                    } else {
                        (None, false, InvokeLocalError::ApplicationNotFound)
                    };

                if let Some(mut invokee) = invokee.take() {
                    if is_application {
                        let arg_start = invokee.payload.as_ptr().wrapping_add(name.len() + 1);
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
                // TODO: handle via Port system
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

    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn setup_registers(&mut self) {
        unsafe {
            orbit_arch::riscv::register::mscratch::write(self as *const Kernel as usize);
            core::arch::asm!("csrr gp, mscratch");
            asm::save_context();
        }
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

    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn handle_register_interrupt(&mut self) {
        let task = self.scheduler.current_mut().unwrap();
        let task_data = task.header.context.tp;
        let interrupt = task.header.context.a1;

        // Find container for the task
        if let Some((index, container)) = self.containers.iter().enumerate().find(|(i, opt)| {
            // such that its data matches task data
            opt.and_then(|cont| cont.data)
                    .and_then(|data| Some(data.data()))
                    .and_then(|data| (data == task_data).then_some(data))
                    .is_some()
                // and driver exists
                && opt.and_then(|cont| cont.driver).is_some()
        }) {
            // and if the ext interrupt controller system is registered
            if let Some(container_index) = self
                .systems
                .find_first_in(SystemId::ExternalInterruptController)
                && let Some(Some(system_driver)) = self.containers.get(container_index)
                && let Some(systems) = system_driver.systems
                && let Some(eic) = systems.external_interrupt_controller
                && let Some(data) = system_driver.data
                && let Some(endpoint) = eic.addrs.get(0)
            {
                // create a task for the system to enable the interrupt
                if let Some(mut new_task) =
                    Task::new(&[], task.header.priority.saturating_sub(1), *endpoint)
                {
                    task.header.context.t0 = 0;
                    new_task.for_driver(data.data());

                    // populating the arguments with
                    // 1 - requesting container index and
                    // 2 - interrupt number
                    new_task.header.context.a1 = index;
                    new_task.header.context.a2 = interrupt;
                    self.scheduler.add(new_task);
                } else {
                    task.header.context.t0 = 1;
                }
            } else {
                // enable interrupt in core and register the
                // container for it
                if interrupt < self.interrupts.len() {
                    self.interrupts[interrupt] = Some(index as u8);
                    self.core.enable_interrupt(interrupt);
                    task.header.context.t0 = 2;
                } else {
                    task.header.context.t0 = 3;
                }
            }
        }
    }

    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn handle_register_system(&mut self) {
        let task = self.scheduler.current_mut().unwrap();
        let task_data = task.header.context.tp;
        let system_id = SystemId::try_from(task.header.context.a1).unwrap();

        // Find container for the task
        if let Some((index, container)) = self.containers.iter().enumerate().find(|(i, opt)| {
            // such that its data matches task data
            opt.and_then(|cont| cont.data)
                    .and_then(|data| Some(data.data()))
                    .and_then(|data| (data == task_data).then_some(data))
                    .is_some()
                // and systems exist
                && opt.and_then(|cont| cont.systems).is_some()
        }) {
            // and insert the container index into systems
            self.systems.insert_in(system_id, index);
            task.header.context.t0 = 0;
        } else {
            task.header.context.t0 = 1;
        }
    }
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn handle_system(&mut self) {
        let system_id = self.scheduler.current().unwrap().header.context.a1;
        let task_data = self.scheduler.current().unwrap().header.context.tp;
        let system_id = SystemId::try_from(system_id).unwrap();

        // Find a container for task
        if let Some((container_index, Some(container))) =
            self.containers.iter().enumerate().find(|(i, opt)| {
                // such that its data matches task data
                opt.and_then(|cont| cont.data)
                    .and_then(|data| Some(data.data()))
                    .and_then(|data| (data == task_data).then_some(data))
                    .is_some()
            })
        {
            let (opt_task, res) = match system_id {
                SystemId::ExternalInterruptController => self.handle_eic_system(container),
                SystemId::Port => self.handle_port_system(container),
                _ => (None, 2),
            };

            self.scheduler.current_mut().unwrap().header.context.t0 = res;
            if let Some(new_task) = opt_task {
                if self.scheduler.current().unwrap().header.context.a0
                    == SysCall::ReturnSystem.discriminant()
                {
                    self.scheduler.pop();
                } else {
                    // Mark current task as blocked by a new one
                    self.scheduler.current_mut().unwrap().header.state =
                        TaskState::Blocked(new_task.header.task_id);
                }

                // Add new task to the scheduler
                self.scheduler.add(new_task);
            } else {
                if self.scheduler.current().unwrap().header.context.a0
                    == SysCall::ReturnSystem.discriminant()
                {
                    self.scheduler.pop();
                }
            }
        } else {
            self.scheduler.current_mut().unwrap().header.context.t0 = 3;
        }
    }

    #[unsafe(no_mangle)]
    #[inline(never)]
    fn handle_eic_system(&self, container: &Container) -> (Option<Pin<Box<Task>>>, usize) {
        let task_syscall = SysCall::from_usize(self.scheduler.current().unwrap().header.context.a0);
        let system_id = self.scheduler.current().unwrap().header.context.a1;
        let container_index = self.scheduler.current().unwrap().header.context.a2;
        let task_data = self.scheduler.current().unwrap().header.context.tp;
        let priority = self.scheduler.current().unwrap().header.priority;

        match task_syscall {
            SysCall::ReturnSystem if container_index == usize::MAX => (None, 2),
            SysCall::ReturnSystem if container_index != usize::MAX => {
                let new_task_id = TASK_ID.get_id();
                if let Some(Some(call_container)) = self.containers.get(container_index)
                    && let Some(call_named) = call_container.named
                    && let Some(call_data) = call_container.data
                    && let Some(call_driver) = call_container.driver
                    && let Some(mut new_task) = Task::new(
                        &[],
                        priority.saturating_add(1),
                        call_driver.interrupt_addr(),
                    )
                {
                    new_task.for_driver(call_data.data());
                    return (Some(new_task), 0);
                } else {
                    return (None, 3);
                }
            }
            SysCall::HandleSystem => {
                // Create new task to handle the interrupt by driver
                let new_task_id = TASK_ID.get_id();
                if let Some(data) = container.data
                    && let Some(systems) = container.systems
                    && let Some(data) = container.data
                    && let Some(mut new_task) = Task::new(
                        &[],
                        priority.saturating_sub(1),
                        systems.external_interrupt_controller.unwrap().addrs[2], // dispatch
                    )
                {
                    new_task.for_driver(data.data());
                    return (Some(new_task), 0);
                } else {
                    return (None, 1);
                }
            }
            _ => (None, 99),
        }
    }

    #[unsafe(no_mangle)]
    #[inline(never)]
    fn handle_port_system(&self, container: &Container) -> (Option<Pin<Box<Task>>>, usize) {
        let mut scheduler = &self.scheduler;
        let task = scheduler.current().unwrap();
        let task_data = task.header.context.tp;
        let task_syscall = SysCall::from_usize(task.header.context.a0);
        let system_id = SystemId::try_from(task.header.context.a1).unwrap();
        let port_task_packet = task.header.context.a2;
        let slice_ptr = task.header.context.a3;
        let slice_len = task.header.context.a4;

        match task_syscall {
            SysCall::HandleSystem => {
                // Create new task to handle the interrupt by driver
                let new_task_id = TASK_ID.get_id();
                if let Some(data) = container.data
                    && let Some(systems) = container.systems
                    && let Some(mut new_task) = Task::new(
                        &[],
                        task.header.priority.saturating_add(1),
                        systems.port.unwrap().addrs[0], // read
                    )
                {
                    new_task.for_driver(data.data());
                    (Some(new_task), 0)
                } else {
                    (None, 1)
                }
            }
            // From read system callback
            // SAFETY: read port system callback has ended,
            // so task priority is the same.
            SysCall::ReturnSystem if slice_ptr != 0 && slice_len != 0 => {
                let slice =
                    unsafe { core::slice::from_raw_parts(slice_ptr as *const u8, slice_len) };
                let new_task_id = TASK_ID.get_id();
                if let Some(data) = container.data
                    && let Some(systems) = container.systems
                    && let Some(mut new_task) = Task::new(
                        slice,
                        task.header.priority,
                        systems.port.unwrap().addrs[1], // write
                    )
                {
                    new_task.for_driver(data.data());
                    (Some(new_task), 0)
                } else {
                    (None, 2)
                }
            }
            // From write system callback
            SysCall::ReturnSystem if slice_ptr == 0 && slice_len == 0 => (None, 3),
            _ => (None, 99),
        }
    }
}
