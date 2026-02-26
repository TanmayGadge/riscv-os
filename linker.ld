OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    . = 0x80200000; /*Set location counter for openSBI*/

    .text : { /*Executable Code*/
        *(.text.entry)
        *(.text .text.*)
    }

    .rodata : {
        *(.rodata .rodata.*)
    }

    .data : {
        *(.data .data.*)
    }

    .bss : {
        *(.bss .bss.stack)
        *(.bss .bss.trap_stack)
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(4096);
        _bss_end = .;
    }

    . = ALIGN(4096); /*Ensure heap starts on a fresh page*/

    _heap_start = .;
}
