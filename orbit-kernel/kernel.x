ENTRY(_start)

SECTIONS
{
    .kernel.text : ALIGN(4)
    {
        /* *(.vector_table.interrupts); */
        /* . = 0x3fc; */
        /* *(.interrupt_handler.*) */
        *(.init);

        . = ALIGN(4);
        *(.kernel.text.setup_event_loop);

        . = ALIGN(4);
        PROVIDE( _handler = .);
        *(.kernel.text.handler);

        . = ALIGN(4);
        *(.kernel.text.interrupt_handler);

        . = ALIGN(4);
        *(.kernel.text.syscall_handler);

        . = ALIGN(4);
        *(.kernel.text.port_handler);

        . = ALIGN(4);
        *(.kernel.text.port_handler_exit);

        . = ALIGN(4);
        *(.kernel.text.context_switch);
        *(.kernel.text);
    } >FLASH

    .kernel.rodata : ALIGN(4)
    {
        *(.kernel.rodata);
    } >FLASH

    .kernel.data : ALIGN(4)
    {
        *(.kernel.data);
    } >RAM AT>FLASH

    .kernel.bss : ALIGN(4)
    {
        *(.bss.* .sbss.*);
        . = ALIGN(4);
    } >RAM AT>FLASH
}
