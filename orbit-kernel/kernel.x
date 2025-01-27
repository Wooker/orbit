/* ENTRY(_start) */

SECTIONS
{
    .kernel : ALIGN(4)
    {
        PROVIDE( _sorbit_kernel = . );
        *( .kernel .kernel.* );
        PROVIDE( _eorbit_kernel = . );
    } >FLASH 

    .apps : ALIGN(4)
    {
        PROVIDE( _sapps = . );
        *( .apps .apps.* );
        PROVIDE( _eapps = . );
    } >FLASH 
}
