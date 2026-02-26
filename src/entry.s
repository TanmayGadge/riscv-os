.section .text.entry
.globl _start
.globl trap_vector

.section .bss.stack
.globl stack_bottom
.globl stack_top

.section .text.entry
_start:
    # Set up the stack pointer.
    la sp, stack_top

    la t0, trap_stack_top

    csrw sscratch, t0

    call kmain
1:
    j 1b



.section .bss.stack

.balign 16 # 16 byte alignment
stack_bottom:
    .space 64 * 1024 
.balign 16 
stack_top:

.section .bss.trap_stack
.globl trap_stack_bottom
.globl trap_stack_top

.balign 16 
trap_stack_bottom:
    .space 4 * 1024

trap_stack_top:

.section .text.entry

.balign 16 
trap_vector:

    csrrw sp, sscratch, sp

    addi sp, sp, -272

    sd ra, 0(sp)
    ; sd sp, 8(sp)
    sd gp,  16(sp)
    sd tp,  24(sp)
    sd t0,  32(sp)
    sd t1,  40(sp)
    sd t2,  48(sp)
    sd s0,  56(sp)
    sd s1,  64(sp)
    sd a0,  72(sp)
    sd a1,  80(sp)
    sd a2,  88(sp)
    sd a3,  96(sp)
    sd a4, 104(sp)
    sd a5, 112(sp)
    sd a6, 120(sp)
    sd a7, 128(sp)
    sd s2, 136(sp)
    sd s3, 144(sp)
    sd s4, 152(sp)
    sd s5, 160(sp)
    sd s6, 168(sp)
    sd s7, 176(sp)
    sd s8, 184(sp)
    sd s9, 192(sp)
    sd s10,200(sp)
    sd s11,208(sp)
    sd t3, 216(sp)
    sd t4, 224(sp)
    sd t5, 232(sp)
    sd t6, 240(sp)

    csrr t0, sepc
    sd t0, 248(sp)
    
    csrr t1, sscratch
    sd t1, 8(sp)

    call trap_handler

    ld ra, 0(sp)
    ; ld sp, 8(sp)
    ld gp,  16(sp)
    ld tp,  24(sp)
    ld t0,  32(sp)
    ld t1,  40(sp)
    ld t2,  48(sp)
    ld s0,  56(sp)
    ld s1,  64(sp)
    ld a0,  72(sp)
    ld a1,  80(sp)
    ld a2,  88(sp)
    ld a3,  96(sp)
    ld a4, 104(sp)
    ld a5, 112(sp)
    ld a6, 120(sp)
    ld a7, 128(sp)
    ld s2, 136(sp)
    ld s3, 144(sp)
    ld s4, 152(sp)
    ld s5, 160(sp)
    ld s6, 168(sp)
    ld s7, 176(sp)
    ld s8, 184(sp)
    ld s9, 192(sp)
    ld s10,200(sp)
    ld s11,208(sp)
    ld t3, 216(sp)
    ld t4, 224(sp)
    ld t5, 232(sp)
    ld t6, 240(sp)

    ld t0, 248(sp)
    csrw sepc, t0

    
    ld t1, 8(sp)
    csrw sscratch, t1
    addi sp, sp, 272

    csrrw sp, sscratch, sp

    sret


