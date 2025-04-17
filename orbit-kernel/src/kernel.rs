use core::{
    arch::{asm, naked_asm},
    mem::MaybeUninit,
    sync::atomic::compiler_fence,
};

use chip::{pac::Peripherals, PORT_PTR};

use orbit_arch::{interface::pmp::Pmp, riscv::register::mtvec, Core, PMP};

use crate::{
    application::{AppContainer, Context, PmpEntry},
    clock::Clocks,
    port::{message::Message, ringbuf::RingBuf, Port, RingbufType, RINGBUF_SIZE},
};

const APPS: usize = 4;

#[used]
#[no_mangle]
#[link_section = ".kernel.rodata"]
pub static KERNEL_MAJOR: u8 = 0;

#[used]
#[no_mangle]
#[link_section = ".kernel.rodata"]
pub static KERNEL_MINOR: u8 = 1;

#[used]
#[no_mangle]
#[link_section = ".kernel.bss"]
pub static mut KERNEL: MaybeUninit<Kernel> = MaybeUninit::uninit();

#[repr(C, align(4))]
pub struct Kernel<'k> {
    context: Context,
    apps: [MaybeUninit<AppContainer<PMP>>; APPS],
    running: usize,
    port: MaybeUninit<Port<'k>>,
    pub(crate) peripherals: MaybeUninit<Peripherals>,
    pub core: Core<PMP>,
    pub clock: Clocks,
}

unsafe impl<'k> Sync for Kernel<'k> {}

impl<'k> Kernel<'k> {
    #[link_section = ".kernel.text"]
    pub const fn new() -> Self {
        Self {
            context: Context::new(),
            peripherals: { MaybeUninit::<Peripherals>::uninit() },
            core: Core::new(),
            port: MaybeUninit::<Port>::uninit(),
            apps: [MaybeUninit::uninit(); APPS],
            clock: Clocks::default(),
            running: 0,
        }
    }

    #[inline(never)]
    #[link_section = ".text"]
    pub fn version(&self) -> (u8, u8) {
        (KERNEL_MAJOR, KERNEL_MINOR)
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn add_application(
        &mut self,
        index: usize,
        app_struct: usize,
        app_main_addr: usize,
        app_interrupt_addr: usize,
        context: Context,
        buf: *mut RingBuf<RINGBUF_SIZE, RingbufType>,
        // peripherals: [Option<KernelPeripherals>; PMP],
    ) {
        let app = unsafe { self.apps.get_unchecked_mut(index) };
        app.write(AppContainer::new(
            context,
            buf,
            [PmpEntry::default(); PMP],
            app_struct,
            app_main_addr,
            app_interrupt_addr,
            [None; PMP], // peripherals,
        ));
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    fn set_pmp(&mut self, app: &AppContainer<PMP>) {
        for (i, pe) in app.get_pmp().iter().enumerate() {
            let _ = self
                .core
                .pmp
                .write_cfg(0, i, pe.range, pe.permission, pe.locked);
            let _ = self.core.pmp.write_addr(i, pe.address);
        }
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn clock(&self) -> u32 {
        self.clock.hclk.raw()
    }

    #[inline(never)]
    #[no_mangle]
    #[link_section = ".kernel.text.port_handler"]
    pub fn port_handler(&mut self) {
        let mut a1: usize;
        unsafe { asm!("mv {}, a1", out(reg) a1) };
        if a1 == 0x100 {
            return;
        }
        compiler_fence(core::sync::atomic::Ordering::SeqCst);

        let port = unsafe { self.port.assume_init_mut() };
        if let Some(mut action) = port.handle() {
            match action.message {
                Message::Invoke => {
                    if let Some(info) = action.rbuf.read() {
                        let app_index = 0;

                        // Get application container
                        let app =
                            unsafe { self.apps.get_unchecked_mut(app_index).assume_init_mut() };

                        // Flush the application buffer
                        app.buf().flush();

                        // Write command arguments after the space to
                        // the application buffer
                        for ch in info.iter().skip_while(|e| **e != b' ').skip(1) {
                            app.buf().push(*ch);
                        }

                        // Run the application
                        unsafe {
                            Self::save_context();
                            self.running = app_index;
                            Self::port_handler_call_app();
                        }

                        // Goes here after application call

                        // Write application output
                        if let Some(output) = app.buf().read() {
                            port.write_str(output);
                        }

                        // Exit the handler
                        unsafe { asm!("la ra, port_handler_exit") };
                    }
                }
                Message::Ok => {}
                Message::Unknown => {
                    port.write_str(b"Unknown message\r");
                }
            }
        }
    }

    #[naked]
    #[no_mangle]
    #[link_section = ".kernel.text"]
    unsafe extern "C" fn port_handler_call_app() {
        naked_asm!(
            "
            la t0, setup_event_loop;
            csrw mepc, t0;
            sw ra, 0x0(gp);
            mv a0, gp;
            mret;
            ",
        );
    }

    #[naked]
    #[no_mangle]
    #[link_section = ".kernel.text.port_handler_exit"]
    unsafe extern "C" fn port_handler_exit() {
        naked_asm!(
            "
            la t0, wait;
            csrw mepc, t0;
            mret;
            ",
        );
    }

    #[inline(never)]
    #[link_section = ".kernel.text"]
    pub fn initialize(&mut self) {
        // self.clock.freeze();
        self.peripherals.write(unsafe { Peripherals::steal() });
        self.port.write(Port::new(unsafe { &*(PORT_PTR) }));
        self.core.pmp.default();
        self.running = 0;
        self.context = Context::new();
        self.context.ra = Self::port_handler_exit as *const fn() as usize;

        // Save kernel context to mscratch
        orbit_arch::riscv::register::mscratch::write(&self.context as *const Context as usize);

        unsafe {
            // Set gp
            asm!("csrr gp, mscratch");
            // Save trap handler
            mtvec::write(
                Self::handler as *const fn() as usize,
                mtvec::TrapMode::Direct,
            );

            // TODO: Move to port init
            // Enable UART4 interrupt
            #[cfg(feature = "ch32v208wbu6")]
            orbit_arch::pfic::enable_interrupt(66);

            #[cfg(feature = "ch32x035")]
            orbit_arch::pfic::enable_interrupt(32);
        }

        unsafe { asm!("mret") };
    }

    #[no_mangle]
    #[inline(never)]
    #[link_section = ".kernel.text"]
    fn wait(&self) {
        loop {}
    }

    #[allow(undefined_naked_function_abi)]
    #[naked]
    #[no_mangle]
    #[link_section = ".kernel.text.main"]
    unsafe fn kernel_main() {
        naked_asm!(
            "
            call setup_event_loop;
            "
        )
    }

    #[no_mangle]
    #[inline(never)]
    #[link_section = ".kernel.text.setup_event_loop"]
    fn setup_event_loop(&mut self) {
        let app_cont = unsafe { self.apps.get_unchecked(self.running).assume_init_read() };
        self.set_pmp(&app_cont);
        unsafe {
            asm!(
                "",
                in("a0") self as *const Kernel as usize,
                in("a1") app_cont.struct_addr(),
                in("a2") app_cont.main_addr(),
                in("a3") app_cont.interrupt_addr(),
            )
        }
        self.context_switch(
            app_cont.struct_addr(),
            app_cont.main_addr(),
            app_cont.interrupt_addr(),
        );
    }

    // Save registers if not _e_ extension
    #[naked]
    #[no_mangle]
    #[cfg(target_feature = "e")]
    #[link_section = ".kernel.text"]
    unsafe extern "C" fn save_context() {
        naked_asm!(
            // "sw ra, 0x0(gp);",
            // Save registers
            "
                    sw sp, 0x4(gp);
                    sw gp, 0x8(gp);
                    sw tp, 0xc(gp);
                    sw t0, 0x10(gp);
                    sw t1, 0x14(gp);
                    sw t2, 0x18(gp);
                    sw s0, 0x1c(gp);
                    sw s1, 0x20(gp);
                    sw a0, 0x24(gp);
                    sw a1, 0x28(gp);
                    sw a2, 0x2c(gp);
                    sw a3, 0x30(gp);
                    sw a4, 0x34(gp);
                    sw a5, 0x38(gp);
                    ",
            "ret"
        );
    }

    // Save registers if not _e_ extension
    #[cfg(not(target_feature = "e"))]
    #[naked]
    #[no_mangle]
    #[link_section = ".kernel.text"]
    unsafe extern "C" fn save_context() {
        naked_asm!(
            // "sw ra, 0x0(gp);",
            // Save registers
            "
                    sw sp, 0x4(gp);
                    sw gp, 0x8(gp);
                    sw tp, 0xc(gp);
                    sw t0, 0x10(gp);
                    sw t1, 0x14(gp);
                    sw t2, 0x18(gp);
                    sw s0, 0x1c(gp);
                    sw s1, 0x20(gp);
                    sw a0, 0x24(gp);
                    sw a1, 0x28(gp);
                    sw a2, 0x2c(gp);
                    sw a3, 0x30(gp);
                    sw a4, 0x34(gp);
                    sw a5, 0x38(gp);
                    sw a6, 0x3c(gp);
                    sw a7, 0x40(gp);
                    sw s2, 0x44(gp);
                    sw s3, 0x48(gp);
                    sw s4, 0x4c(gp);
                    sw s5, 0x50(gp);
                    sw s6, 0x54(gp);
                    sw s7, 0x58(gp);
                    sw s8, 0x5c(gp);
                    sw s9, 0x60(gp);
                    sw s1, 0x64(gp);
                    sw s1, 0x68(gp);
                    sw t3, 0x6c(gp);
                    sw t4, 0x70(gp);
                    sw t5, 0x74(gp);
                    sw t6, 0x78(gp);
                    ",
            "ret"
        );
    }

    #[inline(never)]
    #[no_mangle]
    #[link_section = ".kernel.text.context_switch"]
    fn context_switch(&mut self, _struct_addr: usize, _main_addr: usize, _interrupt_addr: usize) {
        unsafe {
            asm!(
                "
                lw a5, 0x38(gp);
                ",
                // If mcause is interrupt
                // switch to interrupt handler
                "
                csrr t0, mcause;
                la t1, _port_int;
                beq t0, t1, {1};
                srli t0, t0, 31;
                bnez t0, {0};
                ",
                // Switch to app
                "
                j {1}
                ",
                sym switch_to_interrupt,
                sym switch_to_app,
            );

            #[naked]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn switch_to_interrupt() {
                naked_asm!(
                    // Save mepc to a5
                    "
                    csrr a5, mepc;
                    ",
                    // Set mepc to interrupt handler
                    "
                    csrw mepc, a3;
                    ",
                    "
                    li t0, 0;
                    csrw mcause, t0;
                    ",
                    // Set mstatus 0x0
                    "
                    li t0, 0x0;
                    csrw mstatus, t0;
                    ",
                    "
                    call save_context;
                    call load_context;
                    "
                );
            }

            #[naked]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn switch_to_app() {
                naked_asm!(
                    "
                    j {0}
                    // bltz a5, {0};
                    // bgez a5, {1};
                    ",
                    sym switch_to_app_main,
                    sym switch_to_app_from_interrupt
                );
            }

            #[naked]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn switch_to_app_main() {
                naked_asm!(
                    // Set mepc to application main
                    "
                    csrw mepc, a2;
                    ",
                    // Set mstatus 0x80
                    "
                    li t0, 0x80;
                    csrw mstatus, t0;
                    ",
                    // Set a5 to some value
                    "
                    li a5, -1;
                    ",
                    // call save_context;
                    "
                    call load_context;
                    ",
                );
            }

            #[naked]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn switch_to_app_from_interrupt() {
                naked_asm!(
                    // Set mepc to where interrupt has fired
                    "
                    csrw mepc, a5;
                    ",
                    // Set mstatus 0x80
                    "
                    li t0, 0x80;
                    csrw mstatus, t0;
                    ",
                    "
                    call save_context;
                    call load_context;
                    "
                );
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_check() {
                naked_asm!(
                    // Save t0 on stack
                    "
                    addi sp, sp, -0x4;
                    sw t0, 0x0(sp);
                    ",
                    // Use t0 for mscratch
                    "
                    csrr t0, mscratch;
                    ",
                    // Jump to a loading function
                    // t0 = address of returned context
                    "
                    beq t0, gp, load_for_app;
                    bne t0, gp, load_for_kernel;
                    "
                )
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_for_app() {
                naked_asm!(
                    // Save app context to gp
                    "
                    mv gp, a1;
                    ",
                    // Restore t0
                    "
                    lw t0, 0x0(sp);
                    addi sp, sp, 0x4;
                    ",
                    // Return to load_context
                    "
                    ret;
                    "
                )
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_for_kernel() {
                naked_asm!(
                    // Save kernel context to gp
                    "
                    csrr gp, mscratch;
                    ",
                    // Restore t0
                    "
                    lw t0, 0x0(sp);
                    addi sp, sp, 0x4;
                    ",
                    // Return to load_context
                    "
                    ret;
                    "
                )
            }

            // Load registers if not _e_ extension
            #[cfg(target_feature = "e")]
            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_context() {
                naked_asm!(
                    "
                    call load_check;
                    ",
                    // Load registers
                    "
                    lw ra, 0x0(gp);
                    lw gp, 0x8(gp);
                    lw tp, 0xc(gp);
                    lw t0, 0x10(gp);
                    lw t1, 0x14(gp);
                    lw t2, 0x18(gp);
                    lw s0, 0x1c(gp);
                    lw s1, 0x20(gp);
                    lw a0, 0x24(gp);
                    lw a1, 0x28(gp);
                    lw a2, 0x2c(gp);
                    lw a3, 0x30(gp);
                    lw a4, 0x34(gp);
                    lw a5, 0x38(gp);
                    ",
                    "j load_finish"
                );
            }

            // Load registers if not _e_ extension
            #[cfg(not(target_feature = "e"))]
            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_context() {
                naked_asm!(
                    "
                    call load_check;
                    ",
                    // Load registers
                    // except sp
                    "
                    lw ra, 0x0(gp);
                    lw gp, 0x8(gp);
                    lw tp, 0xc(gp);
                    lw t0, 0x10(gp);
                    lw t1, 0x14(gp);
                    lw t2, 0x18(gp);
                    lw s0, 0x1c(gp);
                    lw s1, 0x20(gp);
                    lw a0, 0x24(gp);
                    lw a1, 0x28(gp);
                    lw a2, 0x2c(gp);
                    lw a3, 0x30(gp);
                    lw a4, 0x34(gp);
                    lw a6, 0x3c(gp);
                    lw a7, 0x40(gp);
                    lw s2, 0x44(gp);
                    lw s3, 0x48(gp);
                    lw s4, 0x4c(gp);
                    lw s5, 0x50(gp);
                    lw s6, 0x54(gp);
                    lw s7, 0x58(gp);
                    lw s8, 0x5c(gp);
                    lw s9, 0x60(gp);
                    lw s1, 0x64(gp);
                    lw s1, 0x68(gp);
                    lw t3, 0x6c(gp);
                    lw t4, 0x70(gp);
                    lw t5, 0x74(gp);
                    lw t6, 0x78(gp);
                    ",
                    "j load_finish"
                );
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_finish() {
                naked_asm!(
                    // Save t0 on stack
                    "
                    addi sp, sp, -0x4;
                    sw t0, 0x0(sp);
                    ",
                    // Use t0 for mscratch
                    "
                    csrr t0, mscratch;
                    ",
                    // Jump to a load finishing function
                    "
                    bne t0, gp, load_finish_for_app;
                    beq t0, gp, load_finish_for_kernel;
                    "
                )
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_finish_for_app() {
                naked_asm!(
                    "
                    csrr t0, mepc;
                    bne t0, a5, load_finish_for_app_to_main;
                    beq t0, a5, load_finish_for_app_from_interrupt;
                    "
                )
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_finish_for_app_to_main() {
                naked_asm!(
                    // Restore t0
                    "
                    lw t0, 0x0(sp);
                    addi sp, sp, 0x4;
                    ",
                    // Load sp
                    "
                    lw sp, 0x4(gp);
                    ",
                    // Load a5
                    "
                    lw a5, 0x38(gp);
                    ",
                    "
                    mv a0, gp;
                    ",
                    // Return to mepc location
                    "mret;"
                )
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_finish_for_app_from_interrupt() {
                naked_asm!(
                    // Reset a5 and save
                    "
                    csrr t0, mscratch;
                    li a5, -1;
                    sw a5, 0x38(t0);
                    ",
                    // Restore t0
                    "
                    lw t0, 0x0(sp);
                    addi sp, sp, 0x4;
                    ",
                    // Load sp
                    "
                    lw sp, 0x4(gp);
                    ",
                    // Load a5
                    "
                    lw a5, 0x38(gp);
                    ",
                    // Return to mepc location
                    "mret;"
                )
            }

            #[naked]
            #[no_mangle]
            #[link_section = ".kernel.text"]
            unsafe extern "C" fn load_finish_for_kernel() {
                naked_asm!(
                    // Restore t0
                    "
                    lw t0, 0x0(sp);
                    addi sp, sp, 0x4;
                    ",
                    // Load sp, a5
                    "
                    lw sp, 0x4(gp);
                    lw a5, 0x38(gp);
                    ",
                    // Set a0 to gp (&self)
                    "mv a0, gp;",
                    "
                    li t0, 0x1880;
                    csrw mstatus, t0;
                    ",
                    // "lw t0, wait",
                    "csrw mepc, ra",
                    // Return to ra location
                    "mret; ",
                )
            }
        }
    }

    #[naked]
    #[no_mangle]
    #[link_section = ".kernel.text.handler"]
    unsafe extern "C" fn handler() {
        naked_asm!(
            // Read mcause
            "
            csrr t0, mcause;
            ",
            // Check if it is startup exception number
            "
            li t1, 0x100;
            beq a1, t1, return_handler;
            ",
            // Check if mcause is interrupt
            "
            srli t1, t0, 31;
            bnez t1, handle_int;
            ",
            // If not interrupt mask cause number
            "
            andi t0, t0, 0x7ff;
            ",
            // Check if cause is ecall
            "
            li t1, 8;
            beq t0, t1, user_ecall;
            ",
            // Other cause
            "
            j handle_loop;
            "
        );

        #[naked]
        #[no_mangle]
        #[link_section = ".kernel.text"]
        unsafe extern "C" fn handle_loop() {
            naked_asm!("j handle_loop;");
        }

        #[naked]
        #[no_mangle]
        #[link_section = ".kernel.text"]
        unsafe extern "C" fn handle_int() {
            naked_asm!(
                "
                // csrr t1, mepc;
                la t1, _port_int;
                beq t0, t1, handle_port;
                j kernel_main;
                ",
            );
        }

        #[naked]
        #[no_mangle]
        #[link_section = ".kernel.text"]
        unsafe extern "C" fn handle_port() {
            naked_asm!(
                "
                csrr a0, mscratch;
                la ra, port_handler_exit;
                j port_handler
                ",
            );
        }

        #[naked]
        #[no_mangle]
        #[link_section = ".kernel.text"]
        unsafe extern "C" fn user_ecall() {
            naked_asm!(
                "
                li t0, -1;
                beq a1, t0, user_ecall_int;
                // li t0, 1;
                // beq a1, t0, syscall_delay;
                call save_context;
                call load_context;
                "
            );
        }

        #[naked]
        #[no_mangle]
        #[link_section = ".kernel.text"]
        unsafe extern "C" fn user_ecall_int() {
            naked_asm!("call load_context;");
        }

        #[naked]
        #[no_mangle]
        #[link_section = ".kernel.text"]
        unsafe extern "C" fn return_handler() {
            naked_asm!("ret");
        }
    }
}
