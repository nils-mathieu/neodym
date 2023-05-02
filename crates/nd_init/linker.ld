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
