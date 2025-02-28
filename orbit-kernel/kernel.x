ENTRY(_start)

SECTIONS
{
    .kernel.text : ALIGN(4)
    {
        /* *(.vector_table.interrupts); */
        /* . = 0x3fc; */
        /* *(.interrupt_handler.*) */
        *(.init);
        *(.kernel.text);
        . = ALIGN(4);
        PROVIDE(_handler = .);
        *(.kernel.text.handler);
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
        *(.kernel.bss);
        *(.sbss .sbss.*);
    } >RAM AT>FLASH

}
