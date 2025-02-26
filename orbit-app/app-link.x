/* Taken from */
/* https://github.com/ch32-rs/ch32-metapac/blob/main/src/chips/ch32v208wbu6/memory_x/memory.x */
/* MEMORY */
/* { */
    /* FLASH : ORIGIN = 0x00000000, LENGTH = 4K */
    /* RAM   : ORIGIN = 0x20000000, LENGTH = 2K */
    /* KERNEL_FLASH : ORIGIN = 0x00001000, LENGTH = 1K */
    /* KERNEL_RAM : ORIGIN = 0x20000800, LENGTH = 2K */
    /* FLASH : ORIGIN = 0x00001400, LENGTH = 4K */
    /* RAM : ORIGIN = 0x20001000, LENGTH = 2K */
/* } */

/* ENTRY(main) */

SECTIONS
{
    .apps.text : ALIGN(4)
    {
        *(.*.text);
    } >FLASH

    .rodata.app : ALIGN(4)
    {
        *(.*.rodata);
    } >FLASH

    .data.app : ALIGN(4)
    {
        *(.*.data);
    } >RAM AT>FLASH

    .apps.bss : ALIGN(4)
    {
        *(.*.bss);
    } >RAM AT>FLASH

    .apps.stack : ALIGN(4)
    {
        *(.*.stack);
        PROVIDE( _app_stack_top = .);
    } >RAM AT>FLASH
}
