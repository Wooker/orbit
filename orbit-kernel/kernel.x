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
        PROVIDE(_main = .);
        *(.kernel.text.main);

        . = ALIGN(4);
        PROVIDE(_setup_event_loop = .);
        *(.kernel.text.setup_event_loop);

        . = ALIGN(4);
        PROVIDE(_context_switch = .);
        *(.kernel.text.context_switch);

        . = ALIGN(4);
        PROVIDE(_handler = .);
        *(.kernel.text.handler);

        *(.kernel.text);
    } >FLASH

    .text : ALIGN(4)
    {
        *(.text);
    } > FLASH

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
        *(.kernel.bss);
        *(.sbss .sbss.*);
    } >RAM AT>FLASH

}
