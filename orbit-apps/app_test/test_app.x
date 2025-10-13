SECTIONS
{
	.test_app.text : ALIGN(4)
	{
		*(.text .text.*);
	} > FLASH
}
