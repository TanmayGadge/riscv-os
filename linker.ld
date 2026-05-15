OUTPUT_ARCH(riscv)
ENTRY(phys_start)

KERNEL_OFFSET = 0xFFFFFFFF00000000;
phys_start = _start - KERNEL_OFFSET;

SECTIONS
{
    . = 0x80200000 + KERNEL_OFFSET;

    .text : AT(0x80200000) {
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        *(.rodata .rodata.*)
    }

    .data : AT(ADDR(.data) - KERNEL_OFFSET) {
        *(.data .data.*)
    }

    .bss : AT(ADDR(.bss) - KERNEL_OFFSET) {
        . = ALIGN(8);
        _bss_start = .;
        *(.bss.stack)
        *(.bss.trap_stack)
        *(.bss .bss.*)         
        *(COMMON)
        . = ALIGN(4096);
        _bss_end = .;
    }

    . = ALIGN(4096);
    _heap_start = .;
}