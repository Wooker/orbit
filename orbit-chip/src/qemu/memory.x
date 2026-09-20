/* Taken from */
/* https://github.com/ch32-rs/ch32-metapac/blob/main/src/chips/ch32v208wbu6/memory_x/memory.x */

MEMORY
{
    FLASH : ORIGIN = 0x80000000, LENGTH = 128K
    RAM   : ORIGIN = 0x80020000, LENGTH = 32K
}
REGION_ALIAS("REGION_TEXT", FLASH);
REGION_ALIAS("REGION_RODATA", FLASH);
REGION_ALIAS("REGION_DATA", RAM);
REGION_ALIAS("REGION_BSS", RAM);
REGION_ALIAS("REGION_HEAP", RAM);
REGION_ALIAS("REGION_STACK", RAM);

SECTIONS {
    _sram = ORIGIN(RAM);
    __stack_size = 0x1000;
}
