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

    .kernel.data : ALIGN(4)
    {
        *(.kernel.data);
    } >RAM

    .kernel.bss : ALIGN(4)
    {
        *orbit_arch*.o(.bss .bss.* .sbss.*);
        *chip*.o(.bss .bss* .sbss.*);
        *orbit_kernel*.o(.bss .bss.* .sbss.*);
    } >RAM
}
