use core::arch::{asm, naked_asm};

// TODO: Handle RunApplication enum values
// and call app differently
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn interrupt_handler_exit() {
    naked_asm!(
        // a0 is 0 or 1
        // 1 - call app
        // 0 - don't
        "
            bnez a0, call_app;
            la t0, wait;
            csrw mepc, t0;
            mret;
            ",
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn call_app() {
    naked_asm!(
        "
            la t0, setup_event_loop;
            sw ra, 0x0(gp);
            mv a0, gp;
            j setup_event_loop;
            ",
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn syscall_handler_exit() {
    naked_asm!(
        "

            beqz a1, syscall_handler_return_from_init;
            mv t0, a2;
            li t1, 5;
            beq t0, t1, syscall_handler_await;
            li t1, 1;
            beq t0, t1, syscall_handler_return;
            li t1, 9;
            beq t0, t1, syscall_handler_mem_alloc;
            bnez t0, syscall_handler_return_to_app;
            beqz t0, syscall_handler_return_from_init;
            "
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn syscall_handler_await() {
    naked_asm!(
        "
            csrr t0, mepc;
            addi t0, t0, 4;
            sw t0, 0x7c(a1);
            ",
        "
            la t0, wait;
            csrw mepc, t0;
            mret;
            "
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn syscall_handler_mem_alloc() {
    naked_asm!(
        "
            lw t0, 0x08(a1);
            csrr t1, mepc;
            addi t1, t1, 4;
            csrw mepc, t1;
            j load_context;
            "
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn syscall_handler_return_to_app() {
    naked_asm!(
        "
            lw t0, 0x28(a1);
            csrr t1, mepc;
            addi t1, t1, 4;
            csrw mepc, t1;
            j load_context;
            "
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn syscall_handler_return_from_init() {
    naked_asm!(
        "
            lw ra, 0(gp);
            ret;
            "
    );
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn syscall_handler_return() {
    naked_asm!(
        "
            la t0, wait;
            csrw mepc, t0;
            mret;
            "
    );
}

#[unsafe(no_mangle)]
#[inline(never)]
pub(super) fn wait(_kernel: usize) {
    loop {}
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe fn kernel_main() {
    naked_asm!(
        "
            call setup_event_loop;
            "
    )
}

// Save registers if _e_ extension
#[unsafe(naked)]
#[unsafe(no_mangle)]
#[cfg(target_feature = "e")]
pub unsafe extern "C" fn save_context() {
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
        // Save mepc for the current context
        // regardless it's kernel or application
        "
                    csrr t0, mepc;
                    sw t0, 0x7c(gp);
                    ",
        "ret"
    );
}

// Save registers if not _e_ extension
#[cfg(not(target_feature = "e"))]
#[unsafe(naked)]
#[unsafe(no_mangle)]
pub unsafe extern "C" fn save_context() {
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
        // Save mepc for the current context
        // regardless it's kernel or application
        "
                    csrr t0, mepc;
                    sw t0, 0x7c(gp);
                    ",
        "ret"
    );
}

#[inline(never)]
#[unsafe(no_mangle)]
pub(super) fn context_switch(_kernel: usize, _struct_addr: usize, _addr: usize) {
    unsafe {
        asm!(
            "
                li t0, 0x80;
                csrw mstatus, t0;
                ",
            "csrw mepc, a2;",
            "
                j load_context;
                ",
        );

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_check() {
            naked_asm!(
                "
                    csrr t0, mscratch;
                    ",
                "
                    beq t0, gp, load_for_app;
                    bne t0, gp, load_for_kernel;
                    "
            )
        }

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_for_app() {
            naked_asm!(
                // Save app context to gp
                "mv gp, a1;",
                // Return to load_context
                "ret;"
            )
        }

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_for_kernel() {
            naked_asm!(
                // Save kernel context to gp
                "csrr gp, mscratch;",
                // Return to load_context
                "ret;"
            )
        }

        // Load registers if not _e_ extension
        #[cfg(target_feature = "e")]
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_context() {
            naked_asm!(
                "
                    call load_check;
                    ",
                // Load mepc
                "
                    lw t0, 0x7c(gp);
                    csrw mepc, t0;
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
        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_context() {
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

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_finish() {
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

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_finish_for_app() {
            naked_asm!(
                "
                    j load_finish_for_app_to_main;
                    "
            )
        }

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_finish_for_app_to_main() {
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
                "
                    mv a0, gp;
                    ",
                "mret;"
            )
        }

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_finish_for_app_from_interrupt() {
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
                "mret;"
            )
        }

        #[unsafe(naked)]
        #[unsafe(no_mangle)]
        pub(super) unsafe extern "C" fn load_finish_for_kernel() {
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
                "j handle_mcause;"
            )
        }
    }
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn handler() {
    naked_asm!(
        // Save current context
        "sw ra, 0x0(gp);
            call save_context;
            ",
        // Load kernel context if the trap appeared
        // in an application's context
        "
            csrr t0, mscratch;
            sw a1, 0x28(t0);
            bne t0, gp, load_context;
            ",
        // Handle the trap
        "
            j handle_mcause;
            ",
    )
}

#[unsafe(naked)]
#[unsafe(no_mangle)]
pub(super) unsafe extern "C" fn handle_mcause() {
    naked_asm!(
        // Read mcause
        "
            csrr t0, mcause;
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

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub(super) unsafe extern "C" fn handle_loop() {
        naked_asm!("j handle_loop;");
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub(super) unsafe extern "C" fn handle_int() {
        naked_asm!(
            // t0 = mcause
            // t1 = interrupt bit = 1
            //
            "
                csrr a0, mscratch;
                la ra, interrupt_handler_exit;
                j interrupt_handler;
                ",
        );
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub(super) unsafe extern "C" fn user_ecall() {
        naked_asm!(
            // t0 = exception code
            // t1 = 8 (ecall)
            // t2 = app context address
            "
                csrr a0, mscratch;
                j handle_syscall;
                "
        );
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub(super) unsafe extern "C" fn handle_syscall() {
        naked_asm!(
            "
                la ra, syscall_handler_exit;
                j syscall_handler;
                "
        );
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub(super) unsafe extern "C" fn user_ecall_int() {
        naked_asm!("call load_context;");
    }

    #[unsafe(naked)]
    #[unsafe(no_mangle)]
    pub(super) unsafe extern "C" fn return_handler() {
        naked_asm!("ret");
    }
}
