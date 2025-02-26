SECTIONS
{
    .kernel.text : ALIGN(4)
    {
        *(.init);
        *(.kernel.text);
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
