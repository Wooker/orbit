SECTIONS
{
    .text : ALIGN(4)
    {
        *(.kernel.text .kernel.text.*);
        *(.apps.text);
        *(.text.*);
    } >FLASH

    .rodata : ALIGN(4)
    {
        *(.rodata.*);
    } >FLASH

    .data : ALIGN(4)
    {
        *(.data.*);
    } >RAM AT>FLASH

    .bss : ALIGN(4)
    {
        *(.bss.* .sbss.*);
    } >RAM AT>FLASH

    .kernel.stack ORIGIN(RAM) + LENGTH(RAM) : ALIGN(4)
    {
        *(.kernel.stack);
        PROVIDE( _stack_top = .);
    } >RAM AT>FLASH
}

