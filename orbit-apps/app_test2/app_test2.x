SECTIONS
{
	.apps.app_test2 : ALIGN(4)
	{
		*app_test2*.o(.text .text.*);
	} >FLASH
}
