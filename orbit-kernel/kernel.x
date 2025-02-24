SECTIONS
{
    .kernel.text : ALIGN(4)
    {
        *(.text .text.*)  /* Place all .text symbols here */
    } > KERNEL_FLASH 

    .kernel : ALIGN(4)
    {
        PROVIDE( _sorbit_kernel = . );
        *(.kernel .kernel.*);
        PROVIDE( _eorbit_kernel = . );
    } >KERNEL_RAM AT>KERNEL_FLASH

}

