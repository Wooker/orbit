SECTIONS
{
    .apps : ALIGN(4)
    {
        PROVIDE( _sapps = . );
        *(.apps .apps.*);
        PROVIDE( _eapps = . );
    } >FLASH
} INSERT AFTER .kernel;
