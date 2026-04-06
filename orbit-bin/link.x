SECTIONS
{
    .bin.text : ALIGN(4)
    {
        *(.text.main);
    } >FLASH

    .rust.text : ALIGN(4)
    {
        *(.text .text.*);
    } >FLASH

    .rodata : ALIGN(4)
    {
        *(*.rodata.*);
    } >FLASH

    .data : ALIGN(4)
    {
        *(*.data.*);
    } >RAM AT>FLASH

    .bss : ALIGN(4)
    {
        *(.bss .bss.*);
        *(.sbss .sbss.*);
    } >RAM

    /DISCARD/ : { *(.comment); }
    /DISCARD/ : { *(.eh_frame); }
    /DISCARD/ : { *(.riscv.attributes); }
}

