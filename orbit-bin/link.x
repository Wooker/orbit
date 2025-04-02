SECTIONS
{
    .text : ALIGN(4)
    {
        *(.text.*);
    } >FLASH

    .rodata : ALIGN(4)
    {
        *(.rodata.*);
    } >FLASH

    .data : ALIGN(4)
    {
        *(.data.*);
    } >RAM

    .bss : ALIGN(4)
    {
        *(.bss.* .sbss.*);
    } >RAM

    .kernel.stack ORIGIN(RAM) + LENGTH(RAM) : ALIGN(4)
    {
        *(.kernel.stack);
        PROVIDE( _stack_top = .);
    } >RAM
}

