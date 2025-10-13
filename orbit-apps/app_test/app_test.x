SECTIONS
{
	.apps.app_test : ALIGN(4)
	{
		*app_test*.o(.text .text.*);
	} >FLASH
}

