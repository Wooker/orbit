SECTIONS
{
    .text.apps : ALIGN(4)
    {
        *(.text.apps.*);
        *(.rodata.apps.*);
    } >FLASH

    .bss.apps : ALIGN(4)
    {
        *(.data.apps.*);
        *(.bss.apps.*);
        /* *(.stack.apps.*); */
    } >RAM
}

