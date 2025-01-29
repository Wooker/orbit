/* ENTRY(_start) */

SECTIONS
{
    .kernel : ALIGN(4)
    {
        PROVIDE( _sorbit_kernel = . );
        *( .kernel .kernel.* );
        PROVIDE( _eorbit_kernel = . );
    } >FLASH 
}
