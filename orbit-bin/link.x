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
        *(.rodata.*);
    } >FLASH

    .data : ALIGN(4)
    {
        *(.data.*);
    } >FLASH

    .bss : ALIGN(4)
    {
        *(.bss .bss.*);
        *(.sbss .sbss.*);
    } >RAM AT>FLASH

    .eh_frame :
    {
        *(.eh_frame);
    } > RAM AT>FLASH

    .kernel.stack ORIGIN(RAM) + LENGTH(RAM) - 24-40 : ALIGN(4)
    {
        PROVIDE( _stack_top = .);
    } >RAM

    /DISCARD/ : { *(.comment); }
    /DISCARD/ : { *(.riscv.attributes); }
}

