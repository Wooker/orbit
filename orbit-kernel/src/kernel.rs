pub mod asm;
mod scheduler;

use crate::{
    RINGBUF_SIZE,
    allocator::{ALLOCATOR, SimpleAllocator},
    application_container::{AppContainer, RunApplication},
    claim::KernelPeripherals,
    clock::Clocks,
    context::Context,
    id::ID,
    kernel::scheduler::Scheduler,
    port::{
        Port,
        port_kind::{PORT_INTERRUPTS, PORT_NUM},
    },
    ringbuf::RingBuf,
    syscall::SysCall,
    task::Task,
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
    packet::{HEADER_LEN, Packet},
    types::Flags,
};

unsafe extern "C" {
    static KEEP_LIBOS: extern "C" fn() -> !;
}

pub const APPS: usize = 5;
pub(crate) static PACKET_ID: ID<u16> = ID::new(0);
pub static KERNEL_MAJOR: u8 = 0;
pub static KERNEL_MINOR: u8 = 1;

#[repr(C, align(4))]
pub struct Kernel<'k> {
    context: Context,
    running: Option<usize>,
    apps: [MaybeUninit<AppContainer<'k>>; APPS],
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

unsafe fn init_memory() {
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
            apps: [MaybeUninit::uninit(); APPS],
            claims: [false; KernelPeripherals::MAX as usize],
            clock,
            running: None,
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
    pub fn add_application(&mut self, index: usize, app_cont: AppContainer<'k>, sp: usize) {
        let app_i = unsafe { self.apps.get_unchecked_mut(index) };
        app_i.write(app_cont);
        // self.running = Some(index);
        // app_cont.context().sp = sp;
        // app_cont.context().gp = app_cont.context() as *const Context as usize;
        unsafe {
            asm!("sw ra, 0x0(gp);");
            // asm!("sw sp, 0x4(gp);");
        }
        // self.setup_event_loop(RunApplication::Init);
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
        let awaiting = self
            .ports
            .iter()
            .filter_map(|port| port.awaiting.then(|| port))
            .count();
        let port = &mut self.ports[i];
        port.msg += 1;
        let mut payload_buf = [0; RINGBUF_SIZE - HEADER_LEN];
        if let Some(packet) = port.handle(&mut payload_buf) {
            match packet.msg_type {
                Message::Invoke => {
                    let divider_index =
                        packet.payload.iter().enumerate().find(|(i, b)| **b == b' ');
                    let (name, arg) = divider_index.map_or((packet.payload, None), |index| {
                        let (a, b) = packet.payload.split_at(index.0);
                        (a, Some(b))
                    });

                    let mut out = [0u8; 256];
                    if let Some((app_index, _maybe_app)) =
                        self.apps.iter().enumerate().find(|(_, app)| {
                            let app = unsafe { app.assume_init_read() };
                            let app_name = app.name();
                            app_name.as_bytes().eq(name)
                        })
                    {
                        let app =
                            unsafe { self.apps.get_unchecked_mut(app_index).assume_init_mut() };

                        if let Some(arg) = arg {
                            arg.iter().skip(1).for_each(|b| app.buf().push(*b));
                        }

                        if let Some(task) = Task::new(packet, 1, unsafe { KEEP_LIBOS as usize }) {
                            let task_id = task.task_id;
                            self.running = Some(app_index);
                            self.scheduler.add(task);
                            packet
                                .reply(&task_id.to_le_bytes())
                                .encode(&mut out)
                                .and_then(|size| {
                                    port.send(&out[..size])
                                        .map_err(|e| EncodeError::BufferTooSmall)
                                });
                        } else {
                            packet
                                .reply(b"Could not create task")
                                .encode(&mut out)
                                .and_then(|size| {
                                    port.send(&out[..size])
                                        .map_err(|e| EncodeError::BufferTooSmall)
                                });
                        }
                    } else {
                        packet
                            .reply(b"Unknown name")
                            .encode(&mut out)
                            .and_then(|size| {
                                port.send(&out[..size])
                                    .map_err(|e| EncodeError::BufferTooSmall)
                            });
                    }
                }
                Message::Reply => {
                    if let Some(app_index) = self.running {
                        let info = packet.payload;

                        let app =
                            unsafe { self.apps.get_unchecked_mut(app_index).assume_init_mut() };

                        // Write command arguments after the space to
                        // the application buffer
                        for ch in info.iter() {
                            app.buf().push(*ch);
                        }

                        if awaiting - 1 == 0 {}
                    } else {
                        let mut resp: RingBuf<RINGBUF_SIZE> = RingBuf::new();
                        resp.push(Message::Unknown.into());
                        resp.push(Message::Reply.into());
                        port.write(&resp.buf);
                    }
                }
                Message::KernelVersion => {
                    let mut out = [0u8; 64];
                    let pkt = Packet {
                        version: PROTOCOL_VERSION,
                        flags: Flags::empty(),
                        packet_id: PACKET_ID.get_id(),
                        src: packet.dst,
                        dst: packet.src,
                        ttl: MAX_TTL,
                        msg_type: Message::Reply,
                        payload: &[KERNEL_MAJOR, KERNEL_MINOR],
                    };
                    if let Ok(size) = pkt.encode(&mut out) {
                        let _ = port.send(&out[..size]);
                    }
                }
                _ => {}
            };
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn interrupt_handler(&mut self) {
        let code = orbit_arch::riscv::register::mcause::read().code();

        // Check if it's a port interrupt
        if let Some((index, _)) = PORT_INTERRUPTS
            .0
            .iter()
            .enumerate()
            .find(|(_, (_, interrupt, _))| *interrupt == code)
        {
            // Returns bool to indicate if an application
            // is invoked
            self.port_handler(index);
        } else {
            // TODO: invoke app interrupt
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn syscall_handler(&mut self) {
        let self_addr = self as *const Kernel as usize;
        let app = unsafe { self.apps.get_unchecked_mut(self.running.unwrap_unchecked()) };

        let syscall = SysCall::from_usize(self.scheduler.current().unwrap().context.a0);
        let task_addr =
            self.scheduler.current().unwrap().as_ref().get_ref() as *const Task as usize;
        let task_return = self.scheduler.current().unwrap().context.a0;

        match syscall {
            SysCall::ReturnInit => {
                let app_cont = unsafe { app.assume_init_mut() };
                match app_cont.context().a1 {
                    0 => {
                        self.running = None;
                        unsafe {
                            asm!(
                                "",
                                in("a0") self_addr,
                                in("a1") app_cont.struct_addr() as usize,
                                in("a2") app_cont.main_addr() as usize,
                                in("a3") app_cont.interrupt_addr() as usize,
                            )
                        }
                    }
                    1 => {
                        unsafe {
                            app.assume_init_drop();
                            self.apps[self.running.unwrap_unchecked()] = MaybeUninit::zeroed();
                        }
                        // app.write(unsafe { core::mem::zeroed() });
                        unsafe {
                            asm!(
                                "",
                                in("a0") self_addr,
                                in("a1") 0,
                                in("a2") 0,
                                in("a3") 0,
                            )
                        }
                    }
                    _ => {}
                }
            }
            SysCall::ReturnMain => {
                orbit_arch::riscv::register::mepc::write(asm::wait as *const fn() as usize);
                self.context.a1 = 0;

                let app_cont = unsafe { app.assume_init_mut() };
                self.running = None;
                if let Some(port) = self
                    .ports
                    .iter_mut()
                    .filter_map(|port| port.msg.ne(&0usize).then(|| port))
                    .nth(0)
                {
                    let mut out = [0u8; 96];
                    if let Some(task) = self.scheduler.pop() {
                        task.packet
                            .reply(&task.task_id.to_le_bytes())
                            .encode(&mut out)
                            .and_then(|size| {
                                port.send(&out[..size])
                                    .map_err(|_| EncodeError::BufferTooSmall)
                            });
                    } else {
                        let pkt = Packet {
                            version: PROTOCOL_VERSION,
                            flags: Flags::empty(),
                            packet_id: PACKET_ID.get_id(),
                            src: 0,
                            dst: 0,
                            ttl: MAX_TTL,
                            msg_type: Message::Reply,
                            payload: &[KERNEL_MAJOR, KERNEL_MINOR],
                        };
                        pkt.encode(&mut out).and_then(|size| {
                            port.send(&out[..size])
                                .map_err(|_| EncodeError::BufferTooSmall)
                        });
                    }
                }
                self.ports.iter_mut().for_each(|port| {
                    port.msg = 0;
                });
                unsafe {
                    asm!(
                        "",
                        in("a0") self_addr,
                        in("a1") task_addr,
                        in("a2") task_return,
                        in("a3") app_cont.interrupt_addr() as usize,
                    )
                }
            }
            SysCall::NumPorts => {
                let app_cont = unsafe { app.assume_init_mut() };
                usize::to_le_bytes(PORT_NUM)
                    .iter()
                    .for_each(|b| app_cont.buf().push(*b));
                app_cont.buf().push(b'\0');
                unsafe {
                    asm!(
                        "",
                        in("a0") self_addr,
                        in("a1") app_cont.struct_addr() as usize,
                        in("a2") app_cont.main_addr() as usize,
                        in("a3") app_cont.interrupt_addr() as usize,
                    )
                }
            }
            SysCall::SendAll => {
                let app_cont = unsafe { app.assume_init_mut() };

                for port in self
                    .ports
                    .iter_mut()
                    .filter_map(|port| port.msg.eq(&0usize).then(|| port))
                {
                    port.write(app_cont.buf().read());
                    port.awaiting = true;
                }
                unsafe {
                    asm!(
                        "",
                        in("a0") self_addr,
                        in("a1") app_cont.struct_addr() as usize,
                        in("a2") app_cont.main_addr() as usize,
                        in("a3") app_cont.interrupt_addr() as usize,
                    )
                }
            }
            SysCall::Await => {
                // let app_cont = unsafe { app.assume_init_mut() };
                // app_cont.buf().flush();
                // let awaiting_num = self
                //     .ports
                //     .iter()
                //     .filter(|p| unsafe { p.assume_init_read() }.awaiting)
                //     .count();
                // app_cont.buf().push(awaiting_num as u8);
                // app_cont.buf().push(b'\0');
            }
            SysCall::AwaitAll => {
                // let app_cont = unsafe { app.assume_init_mut() };
                // app_cont.buf().flush();
                // for port in self.ports.iter_mut().filter_map(|p| {
                //     let port = unsafe { p.assume_init_mut() };
                //     port.msg.eq(&0usize).then(|| port)
                // }) {
                //     port.write(info);
                //     port.awaiting = true;
                // }
                // app_cont.buf().push(awaiting_num as u8);
                // app_cont.buf().push(b'\0');
            }
            SysCall::MemAlloc => {
                let app_cont = unsafe { app.assume_init_mut() };
                let output = if let Ok(byte_arr) = app_cont.buf().read().try_into() {
                    let heap = app_cont.heap();
                    let alloc_size = usize::from_le_bytes(byte_arr);
                    if let Some((i, space)) =
                        heap.iter().enumerate().skip_while(|(_, p)| **p != 0).next()
                    {
                        if heap[i..].len() > alloc_size {
                            space.to_le_bytes()
                        } else {
                            3_usize.to_le_bytes()
                        }
                    } else {
                        2_usize.to_le_bytes()
                    }
                } else {
                    1_usize.to_le_bytes()
                };

                let app_buf = app_cont.buf();
                output.iter().for_each(|b| app_buf.push(*b));

                unsafe {
                    asm!(
                        "",
                        in("a0") self_addr,
                        in("a1") app_cont.struct_addr() as usize,
                        in("a2") app_cont.main_addr() as usize,
                        in("a3") app_cont.interrupt_addr() as usize,
                    )
                }
            }
            SysCall::ClaimPeripheral => {
                let app_cont = unsafe { app.assume_init_mut() };
                let app_buf = app_cont.buf();
                let ind = app_buf.read().get(0).unwrap().clone() as usize;

                let claim_spot = self.claims.get_mut(ind).unwrap();
                if *claim_spot == false {
                    app_buf.push(1);
                    *claim_spot = true;
                } else {
                    app_buf.push(0);
                }
                unsafe {
                    asm!(
                        "",
                        in("a0") self_addr,
                        in("a1") app_cont.struct_addr() as usize,
                        in("a2") app_cont.main_addr() as usize,
                        in("a3") app_cont.interrupt_addr() as usize,
                    )
                }
            }
            SysCall::Invoke => {
                let app_cont = unsafe { app.assume_init_mut() };
                let app_buf = app_cont.buf();
                let ind = app_buf.read().get(0).unwrap().clone() as usize;

                let claim_spot = self.claims.get_mut(ind).unwrap();
                if *claim_spot == false {
                    app_buf.push(1);
                    *claim_spot = true;
                } else {
                    app_buf.push(0);
                }
                unsafe {
                    asm!(
                        "",
                        in("a0") self_addr,
                        in("a1") app_cont.struct_addr() as usize,
                        in("a2") app_cont.main_addr() as usize,
                        in("a3") app_cont.interrupt_addr() as usize,
                    )
                }
            }
            _ => {
                let app_cont = unsafe { app.assume_init_mut() };
                for port in self
                    .ports
                    .iter_mut()
                    .filter_map(|port| port.msg.eq(&0usize).then(|| port))
                {
                    port.write(&[syscall.discriminant() as u8]);
                    unsafe {
                        asm!(
                            "",
                            in("a0") self_addr,
                            in("a1") app_cont.struct_addr() as usize,
                            in("a2") app_cont.main_addr() as usize,
                            in("a3") app_cont.interrupt_addr() as usize,
                        )
                    }
                }
            }
        }

        // self.set_pmp(&app_cont);
    }

    #[unsafe(no_mangle)]
    #[inline(never)]
    fn setup_event_loop(&mut self) -> ! {
        if let Some(task) = self.scheduler.current() {
            let task_addr = task.as_ref().get_ref() as *const Task as usize;
            let addr = task.context.mepc;
            // self.set_pmp(&app_cont);
            unsafe { asm::context_switch(self as *const Kernel as usize, task_addr, addr) }
        } else {
            unsafe { asm::interrupt_handler_exit() }
        }
    }
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn handle_panic(&mut self, panic_info: &PanicInfo) {
        for port in self.ports.iter_mut() {
            // let msg = panic_info.message().as_str().unwrap();
            // for chunk in msg.as_bytes().chunks(32) {
            //     port.write(chunk);
            // }
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
}
