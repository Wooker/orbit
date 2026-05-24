ENTRY(_start)

SECTIONS
{
    .kernel.text : ALIGN(4)
    {
        *orbit_kernel*.o(.init);
        *orbit_arch*.o(.text .text.*);
        *chip*.o(.text .text.*);
        *orbit_kernel*.o(.text .text.*);
        *orbit_bin*.o(.text .text.*);
    } >FLASH

    .kernel.rodata : ALIGN(4)
    {
        *orbit_kernel*.o(.rodata .rodata.*);
    } >FLASH

    _heap_size = __heap_size;
    .kernel.stack ORIGIN(RAM) + _stack_size : ALIGN(4)
    {
        PROVIDE(_stack_top = .);
    } > RAM

    /* FLASH load address of .data */
    _sidata = LOADADDR(.kernel.data);

    /* RAM runtime addresses of .data */
    _sdata = ADDR(.kernel.data);
    _edata = ADDR(.kernel.data) + SIZEOF(.kernel.data);

    .kernel.data : ALIGN(4)
    {
        *orbit_kernel*(.data, .data.*);
    } >RAM AT>FLASH

    .kernel.bss : ALIGN(4)
    {
        _sbss = .;
        *orbit_arch*.o(.bss .bss.* .sbss.*);
        *chip*.o(.bss .bss* .sbss.*);
        *orbit_kernel*.o(.bss .bss.* .sbss.*);
        _ebss = ORIGIN(RAM)+LENGTH(RAM);
    } >RAM
}
