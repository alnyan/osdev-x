ENTRY(_start);

SECTIONS {
    . = 0x400000;

    .text : {
        *(.text*)
    }

    . = ALIGN(4K);

    .rodata : {
        *(.rodata*)
    }

    . = ALIGN(4K);

    .bss : {
        *(COMMON)
        *(.bss*)
    }
}
