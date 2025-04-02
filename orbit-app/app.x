SECTIONS
{
    .text.apps : ALIGN(4)
    {
        *(.text.apps.*);
        *(.rodata.apps.*);
    } >FLASH

    .apps : ALIGN(4)
    {
        *(.data.apps.*);
        *(.bss.apps.*);
        /* *(.stack.apps.*); */
    } >RAM
}

