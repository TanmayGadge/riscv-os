    .section .text.entry
    .globl _start

_start:
    # Set up the stack pointer.
    # Reserve 64 KB for the stack (you can change this later).
    la sp, stack_top

    # Jump to Rust kernel entry
    call kmain

    # If kmain ever returns (it shouldn't), just loop here forever.
1:
    j 1b


    # -------------------------
    # Allocate a simple stack
    # -------------------------
    .section .bss.stack
    .globl stack_bottom
    .globl stack_top

    # 64 KB stack
    .align 16
stack_bottom:
    .space 64 * 1024
stack_top:
