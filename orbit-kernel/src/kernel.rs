// While compiling with  rustc 1.91.0-nightly (54c581243 2025-08-25)
// cargo produces:
// ```
// warning: `#[unsafe(link_section)]` attribute cannot be used on inherent methods
// ```
// If such behavior is no longer observable on newer versions of rustc,
// remove this attribute
#![allow(unused_attributes)]

mod asm;

use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
};

// use chip::pac::Peripherals;
use orbit_arch::{interface::pmp::Pmp, Core, PMP};

use crate::{
    application_container::{AppContainer, RunApplication},
    claim::KernelPeripherals,
    clock::Clocks,
    context::Context,
    message::Message,
    port::{
        port_kind::{PORT_INTERRUPTS, PORT_NUM},
        Port,
    },
    ringbuf::{RingBuf, TraitBound},
    syscall::SysCall,
    RingbufType, RINGBUF_SIZE,
};

pub const APPS: usize = 5;

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
    apps: [MaybeUninit<AppContainer<'k>>; APPS],
    running: Option<usize>,
    ports: [Port<'k>; PORT_NUM],
    // pub peripherals: Peripherals,
    pub core: Core<PMP>,
    pub clock: Clocks,
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

        // Initialize context
        let context = Context::new();
        // context.ra = Self::wait as *const fn() as usize;

        // Save trap handler
        unsafe {
            crate::arch::riscv::register::mtvec::write(
                asm::handler as *const fn() as usize,
                crate::arch::riscv::register::mtvec::TrapMode::Direct,
            )
        };

        Self {
            context,
            // peripherals: unsafe { Peripherals::steal() },
            core: Core::new(),
            ports,
            apps: [MaybeUninit::uninit(); APPS],
            clock,
            running: None,
        }
    }

    #[inline(never)]
    #[unsafe(link_section = ".text")]
    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }

    #[inline(never)]
    pub const fn add_application(&mut self, index: usize, app_cont: AppContainer<'k>) {
        let app_i = unsafe { self.apps.get_unchecked_mut(index) };
        app_i.write(app_cont);
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
        if let Some(mut action) = port.handle() {
            if port.msg > 0 {
                port.write_str(&[Message::Busy.into(), 0]);
            } else {
                port.msg += 1;
                match action.message {
                    Message::Invoke => {
                        if port.awaiting {}

                        let info = unsafe { action.rbuf.read().unwrap_unchecked() };
                        let (name, arg) = if let Some((name, arg)) =
                            info.split_once(|p| *p == <RingbufType as TraitBound>::termination())
                        {
                            (name, Some(arg))
                        } else {
                            (info, None)
                        };

                        // let mut dbg: RingBuf<RINGBUF_SIZE, RingbufType> = RingBuf::default();
                        // name.iter().for_each(|d| dbg.push(*d));
                        // port.write_str(&str::from_utf8(&dbg.buf).unwrap().trim().as_bytes());
                        // dbg.flush();
                        // port.write_str(&dbg.buf);

                        // Find app by name
                        if let Some((app_index, _)) =
                            self.apps.iter().enumerate().find(|(_, app)| unsafe {
                                app.assume_init_read().name().as_bytes().eq(name)
                            })
                        {
                            // Get the app container
                            let app =
                                unsafe { self.apps.get_unchecked_mut(app_index).assume_init_mut() };

                            if let Some(arg) = arg {
                                // arg.iter().for_each(|d| dbg.push(*d));
                                // port.write_str(&dbg.buf);
                                // dbg.flush();

                                // Flush the application buffer
                                let app_buf = app.buf();
                                app_buf.flush();

                                let mut arg_buf: RingBuf<RINGBUF_SIZE, RingbufType> =
                                    RingBuf::default();
                                // Write command arguments after the space to
                                // the application buffer
                                arg.iter().for_each(|ch| arg_buf.push(*ch));
                                app_buf.buf.copy_from_slice(&arg_buf.buf);
                                app_buf.end = RINGBUF_SIZE;
                                // port.write_str(&app_buf.buf);
                            }

                            // Run the application
                            self.running = Some(app_index);
                            return RunApplication::Main;
                        } else {
                            let mut resp: RingBuf<RINGBUF_SIZE, RingbufType> = RingBuf::default();
                            resp.push(Message::Unknown.into());
                            resp.push(Message::Invoke.into());
                            name.iter().for_each(|b| resp.push(*b));
                            port.write_str(&resp.buf);
                            port.msg -= 1;
                        }
                    }
                    Message::Reply => {
                        if let Some(app_index) = self.running {
                            port.awaiting = false;
                            port.msg = 0;

                            let info = unsafe { action.rbuf.read().unwrap_unchecked() };

                            let app =
                                unsafe { self.apps.get_unchecked_mut(app_index).assume_init_mut() };
                            // Flush the application buffer
                            app.buf().flush();

                            // Write command arguments after the space to
                            // the application buffer
                            for ch in info.iter() {
                                app.buf().push(*ch);
                            }

                            if awaiting - 1 == 0 {
                                return RunApplication::Jumped;
                            }
                        } else {
                            let mut resp: RingBuf<RINGBUF_SIZE, RingbufType> = RingBuf::default();
                            resp.push(Message::Unknown.into());
                            resp.push(Message::Reply.into());
                            port.write_str(&resp.buf);
                            port.msg -= 1;
                        }
                    }
                    Message::Busy => {
                        if let Some(_) = self.running {
                            // return RunApplication::Abort;
                        }
                        port.msg -= 1;
                    }
                    Message::Append => {
                        let info = unsafe { action.rbuf.read().unwrap_unchecked() };
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
                            port.msg -= 1;
                        } else {
                            port.write_str(&[Message::Unknown.into(), Message::Append.into(), 0]);
                            port.msg -= 1;
                        }
                    }
                    Message::Unknown => {
                        port.msg = 0;
                    }
                }
            }
        }
        // let end = port.rbuf.end;
        // port.write(end as u8);
        RunApplication::None
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
    fn syscall_handler(&mut self, syscall: SysCall) {
        {
            let app_cont = unsafe {
                self.apps
                    .get_unchecked_mut(self.running.unwrap_unchecked())
                    .assume_init_mut()
            };

            match syscall.clone() {
                SysCall::Return => {
                    orbit_arch::riscv::register::mepc::write(asm::wait as *const fn() as usize);
                    self.context.a1 = 0;

                    self.running = None;
                    if let Some(port) = self
                        .ports
                        .iter_mut()
                        .filter_map(|port| port.msg.ne(&0usize).then(|| port))
                        .nth(0)
                    {
                        app_cont.buf().fill();
                        if let Some(output) = app_cont.buf().read() {
                            port.write_str(output);
                        }
                    }
                    self.ports.iter_mut().for_each(|port| {
                        port.msg = 0;
                    });
                }
                SysCall::NumPorts => {
                    app_cont.buf().flush();
                    usize::to_le_bytes(PORT_NUM)
                        .iter()
                        .for_each(|b| app_cont.buf().push(*b));
                    app_cont.buf().push(b'\0');
                }
                SysCall::SendAll => {
                    if let Some(info) = app_cont.buf().read() {
                        for port in self
                            .ports
                            .iter_mut()
                            .filter_map(|port| port.msg.eq(&0usize).then(|| port))
                        {
                            port.write_str(info);
                            port.awaiting = true;
                        }
                    }
                }
                SysCall::Await => {
                    // app_cont.buf().flush();
                    // let awaiting_num = self
                    //     .ports
                    //     .iter()
                    //     .filter(|p| unsafe { p.assume_init_read() }.awaiting)
                    //     .count();
                    // app_cont.buf().push(awaiting_num as u8);
                    // app_cont.buf().push(b'\0');
                }
                SysCall::ReceiveAll => {
                    // app_cont.buf().flush();
                    // for port in self.ports.iter_mut().filter_map(|p| {
                    //     let port = unsafe { p.assume_init_mut() };
                    //     port.msg.eq(&0usize).then(|| port)
                    // }) {
                    //     port.write_str(info);
                    //     port.awaiting = true;
                    // }
                    // app_cont.buf().push(awaiting_num as u8);
                    // app_cont.buf().push(b'\0');
                }
                SysCall::ClaimPeripheral => {
                    app_cont.buf().flush();
                }
                _ => {
                    for port in self
                        .ports
                        .iter_mut()
                        .filter_map(|port| port.msg.eq(&0usize).then(|| port))
                    {
                        let start = app_cont.buf().start;
                        let end = app_cont.buf().end;
                        port.write_str(&[start as u8, end as u8, syscall.discriminant() as u8]);
                    }
                }
            }
        }

        let app_cont = unsafe {
            self.apps
                .get_unchecked(self.running.unwrap_unchecked())
                .assume_init_read()
        };
        // self.set_pmp(&app_cont);
        unsafe {
            asm!(
                "",
                in("a0") self as *const Kernel as usize,
                in("a1") app_cont.struct_addr(),
                in("a2") app_cont.main_addr(),
                in("a3") app_cont.interrupt_addr(),
            )
        }
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
