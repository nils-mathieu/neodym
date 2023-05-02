OUTPUT_FORMAT(elf64-x86-64)
OUTPUT_ARCH(x86-64)
ENTRY(entry_point)

SECTIONS
{
    . = 0;

    .text : {
        *(.entry_point)
        *(.text .text.*)
    } :text

    .data : {
        *(.data .data.*)
    } :data

    .rodata : {
        *(.rodata .rodata.*)
    } :rodata

    .bss : {
        *(COMMON)
        *(.bss .bss.*)
    } :data
}
