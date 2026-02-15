// While compiling with  rustc 1.91.0-nightly (54c581243 2025-08-25)
// cargo produces:
// ```
// warning: `#[unsafe(link_section)]` attribute cannot be used on inherent methods
// ```
// If such behavior is no longer observable on newer versions of rustc,
// remove this attribute
#![allow(unused_attributes)]

pub mod asm;
mod port_handler;

use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
    panic::PanicInfo,
};

use alloc::{collections::linked_list::LinkedList, vec::Vec};
// use chip::pac::Peripherals;
use orbit_arch::{Core, PMP};
use spaceport::{
    constants::PROTOCOL_VERSION,
    message::Message,
    packet::{HEADER_LEN, Packet},
    types::Flags,
};

use crate::{
    RINGBUF_SIZE,
    application_container::{AppContainer, RunApplication},
    claim::KernelPeripherals,
    clock::Clocks,
    context::Context,
    kernel::port_handler::handle_invoke,
    port::{
        Port,
        port_kind::{PORT_INTERRUPTS, PORT_NUM},
    },
    ringbuf::RingBuf,
    syscall::SysCall,
};

pub const APPS: usize = 5;
pub(crate) static mut PACKET_ID: usize = 0;
pub(crate) static mut TASK_ID: usize = 0;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".kernel.rodata")]
pub static KERNEL_MAJOR: u8 = 0;

#[used]
#[unsafe(no_mangle)]
#[unsafe(link_section = ".kernel.rodata")]
pub static KERNEL_MINOR: u8 = 1;

#[repr(C, align(4))]
pub struct Kernel<'k> {
    context: Context,
    running: Option<usize>,
    apps: [MaybeUninit<AppContainer<'k>>; APPS],
    ports: [Port<'k>; PORT_NUM],
    // pub peripherals: Peripherals,
    claims: [bool; KernelPeripherals::MAX as usize],
    pub core: Core<PMP>,
    pub clock: Clocks,
    tasks: Vec<usize>,
}

impl<'k> Kernel<'k> {
    #[rustc_align(4)]
    #[inline(never)]
    pub fn new() -> Self {
        // Enable clocks
        let mut clock = Clocks::default();
        clock.freeze();

        // Initialize ports
        let ports: [Port; PORT_NUM] = core::array::from_fn(|i| {
            let (ptr, interrupt, kind) = PORT_INTERRUPTS.0[i];
            unsafe {
                orbit_arch::pfic::enable_interrupt(interrupt as u8);
            }
            Port::new(unsafe { &*ptr }, kind)
        });
        let ll: Vec<usize> = Vec::new();

        // Save trap handler
        unsafe {
            crate::arch::riscv::register::mtvec::write(
                asm::handler as *const fn() as usize,
                crate::arch::riscv::register::mtvec::TrapMode::Direct,
            )
        };

        let mut kernel = Self {
            context: Context::new(),
            // peripherals: unsafe { Peripherals::steal() },
            core: Core::new(),
            ports,
            apps: [MaybeUninit::uninit(); APPS],
            claims: [false; KernelPeripherals::MAX as usize],
            clock,
            running: None,
            tasks: ll,
        };

        unsafe {
            PACKET_ID = 0;
            TASK_ID = 0;
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
        self.running = Some(index);
        app_cont.context().sp = sp;
        app_cont.context().gp = app_cont.context() as *const Context as usize;
        unsafe {
            asm!("sw ra, 0x0(gp);");
            // asm!("sw sp, 0x4(gp);");
        }
        self.setup_event_loop(RunApplication::Init);
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
    fn port_handler(&mut self, i: usize) -> RunApplication {
        let awaiting = self
            .ports
            .iter()
            .filter_map(|port| port.awaiting.then(|| port))
            .count();
        let port = &mut self.ports[i];
        port.msg += 1;
        let mut payload_buf = [0; RINGBUF_SIZE - HEADER_LEN];
        if let Some(packet) = port.handle(&mut payload_buf) {
            let ra = match packet.msg_type {
                Message::Invoke => {
                    if let Ok(t) = handle_invoke(packet.payload, &mut self.apps, &mut self.running)
                    {
                        unsafe {
                            self.tasks.push(TASK_ID);
                            TASK_ID += 1;
                        }
                        t
                    } else {
                        let mut out = [0u8; 64];
                        let pkt = Packet {
                            version: PROTOCOL_VERSION,
                            flags: Flags::empty(),
                            packet_id: unsafe { PACKET_ID } as u16,
                            src: 0,
                            dst: 0,
                            ttl: 0,
                            msg_type: Message::Unknown,
                            payload: &self.tasks.len().to_le_bytes(),
                        };
                        if let Ok(size) = pkt.encode(&mut out) {
                            let _ = port.send(&out[..size]);
                            unsafe { PACKET_ID += 1 };
                        } else {
                            port.write(b"No output");
                        }
                        RunApplication::None
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

                        if awaiting - 1 == 0 {
                            return RunApplication::Jumped;
                        }
                    } else {
                        let mut resp: RingBuf<RINGBUF_SIZE> = RingBuf::new();
                        resp.push(Message::Unknown.into());
                        resp.push(Message::Reply.into());
                        port.write(&resp.buf);
                    }
                    RunApplication::None
                }
                Message::Busy => {
                    if let Some(_) = self.running {
                        // return RunApplication::Abort;
                    }
                    RunApplication::None
                }
                Message::Append => {
                    let info = packet.payload;
                    let delimiter = info.iter().take_while(|e| **e != b' ').count();
                    let (name, arg) = info.split_at(delimiter);

                    // Find app by name
                    if let Some((app_index, _)) =
                        self.apps.iter().enumerate().find(|(_, app)| unsafe {
                            app.assume_init_read().name().as_bytes().eq(name)
                        })
                    {
                        // Get the app container
                        let app =
                            unsafe { self.apps.get_unchecked_mut(app_index).assume_init_mut() };

                        // Write command arguments after the space to
                        // the application buffer
                        for ch in arg.iter().skip(1) {
                            app.buf().push(*ch);
                        }
                        RunApplication::None
                    } else {
                        port.write(&[Message::Unknown.into(), Message::Append.into(), 0]);
                        RunApplication::None
                    }
                }
                Message::Error => RunApplication::None,
                Message::Unknown => RunApplication::None,
            };
            ra
        } else {
            RunApplication::None
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn interrupt_handler(&mut self) -> RunApplication {
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
            self.port_handler(index)
        } else {
            // TODO: invoke app interrupt
            RunApplication::None
        }
    }

    #[inline(never)]
    #[unsafe(no_mangle)]
    fn syscall_handler(&mut self) {
        let self_addr = self as *const Kernel as usize;
        let app = unsafe { self.apps.get_unchecked_mut(self.running.unwrap_unchecked()) };

        let mut maybe_syscall: MaybeUninit<SysCall> = MaybeUninit::uninit();

        maybe_syscall.write(SysCall::from_usize(
            unsafe { app.assume_init_mut() }.context().a0,
        ));
        let syscall = unsafe { maybe_syscall.assume_init() };

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
                    let pkt = Packet {
                        version: PROTOCOL_VERSION,
                        flags: Flags::empty(),
                        packet_id: unsafe { PACKET_ID } as u16,
                        src: 0,
                        dst: 0,
                        ttl: 0,
                        msg_type: Message::Reply,
                        payload: app_cont.buf().read(),
                    };
                    if let Ok(size) = pkt.encode(&mut out) {
                        let _ = port.send(&out[..size]);
                        unsafe { PACKET_ID += 1 };
                    } else {
                        port.write(b"No output");
                    }
                }
                self.tasks.pop();
                self.ports.iter_mut().for_each(|port| {
                    port.msg = 0;
                });
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
    fn setup_event_loop(&mut self, variant: RunApplication) {
        let app_cont = unsafe {
            self.apps
                .get_unchecked(self.running.unwrap_unchecked())
                .assume_init_read()
        };
        // self.set_pmp(&app_cont);
        let addr = match variant {
            RunApplication::Init => app_cont.init_addr(),
            RunApplication::Main => app_cont.main_addr(),
            RunApplication::Interrupt => app_cont.interrupt_addr(),
            RunApplication::Jumped => app_cont.context().mepc,
            _ => 0,
        };
        unsafe {
            asm!(
                "",
                in("a0") self as *const Kernel as usize,
                in("a1") app_cont.struct_addr(),
                in("a2") addr,
            )
        }
        asm::context_switch(self as *const Kernel as usize, app_cont.struct_addr(), addr);
    }
    #[unsafe(no_mangle)]
    #[inline(never)]
    pub fn handle_panic(&mut self, panic_info: &PanicInfo) {
        for port in self.ports.iter_mut() {
            let msg = panic_info.message().as_str().unwrap();
            for chunk in msg.as_bytes().chunks(32) {
                port.write(chunk);
            }
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
