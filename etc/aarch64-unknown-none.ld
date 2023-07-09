ENTRY(__aarch64_entry);

KERNEL_PHYS_BASE = 0x40080000;
KERNEL_VIRT_OFFSET = 0xFFFFFF8000000000;

SECTIONS {
    . = KERNEL_PHYS_BASE;

    .text.entry : {
        *(.text.entry)
    }

    .data.tables : {
        *(.data.tables)
    }

    . = ALIGN(16);
    . = . + KERNEL_VIRT_OFFSET;

    .text : AT(. - KERNEL_VIRT_OFFSET) {
        *(.text*)
    }

    . = ALIGN(4K);
    .rodata : AT(. - KERNEL_VIRT_OFFSET) {
        *(.eh_frame*)
        *(.rodata*)
    }

    . = ALIGN(4K);
    .data : AT(. - KERNEL_VIRT_OFFSET) {
        *(.data*)
        /* *(.got*) */
    }

    . = ALIGN(4K);
    PROVIDE(__bss_start_phys = . - KERNEL_VIRT_OFFSET);
    .bss : AT(. - KERNEL_VIRT_OFFSET) {
        *(COMMON)
        *(.bss*)
    }
    . = ALIGN(4K);
    PROVIDE(__bss_end_phys = . - KERNEL_VIRT_OFFSET);

    PROVIDE(__kernel_size = . - KERNEL_VIRT_OFFSET - KERNEL_PHYS_BASE);
};
