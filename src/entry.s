.section .text.entry
.globl _start
.globl trap_vector

_start:
    # 1. Force the MMU off completely. This ensures 'satp == 0' works safely.
    csrw satp, zero

    # 2. Zero out the .bss section so Rust spinlocks don't start pre-locked with garbage.
    la   t0, _bss_start
    la   t1, _bss_end
1:
    bge  t0, t1, 2f
    sd   zero, 0(t0)
    addi t0, t0, 8
    j    1b
2:

    # 3. 'la' automatically calculates the PHYSICAL address here
    la   sp, stack_top
    la   s0, trap_stack_top
    csrw sscratch, s0

    call kinit                   # returns root PPN in a0
    mv   s1, a0                  # s1 = root page table physical address

    # Enable MMU
    srli t0, s1, 12
    li   t1, 8
    slli t1, t1, 60
    or   t0, t0, t1
    csrw satp, t0
    sfence.vma

    # ── CRITICAL: jump to the virtual address of the next instruction ──────
    la   t0, 1f                  # t0 = PHYSICAL address of label 1
    
    # Calculate KERNEL_OFFSET (0xFFFFFFFF00000000) into t1
    li   t1, 1
    slli t1, t1, 32
    neg  t1, t1                  
    
    # Add the offset to get the VIRTUAL address!
    add  t0, t0, t1              
    jr   t0                      # Jump! CPU is now executing in high memory.
    
1:
    # From here the PC is virtual. Now fix up sp and sscratch.
    add  sp, sp, t1              # sp → virtual
    add  s0, s0, t1              # s0 → virtual trap stack top
    csrw sscratch, s0

    mv   a0, s1                  # pass root_ptr to kmain
    la   t0, kmain
    jr   t0

2:
    j 2b

# ── Stacks ────────────────────────────────────────────────────────────────
.section .bss.stack
.globl stack_bottom
.globl stack_top

.balign 16
stack_bottom:
    .space 64 * 1024
.balign 16
stack_top:


.section .bss.trap_stack
.globl trap_stack_bottom
.globl trap_stack_top

.balign 16
trap_stack_bottom:
    .space 64 * 1024
.balign 16
trap_stack_top:


# ── Trap vector ───────────────────────────────────────────────────────────
.section .text.entry
.balign 4                        # stvec requires 4-byte alignment (Direct mode)

trap_vector:
    # Atomically swap sp ↔ sscratch.
    # After this: sp = trap stack, sscratch = interrupted sp.
    csrrw sp, sscratch, sp

    addi  sp, sp, -272           # allocate TrapContext (34 registers × 8 bytes)

    # Save t0 first (we need it as scratch immediately).
    sd    t0,  32(sp)

    # Read the original sp out of sscratch and save it.
    csrr  t0, sscratch
    sd    t0,   8(sp)            # TrapContext.sp = interrupted stack pointer

    # Save all other registers.
    sd    ra,   0(sp)
    sd    gp,  16(sp)
    sd    tp,  24(sp)
    sd    t1,  40(sp)
    sd    t2,  48(sp)
    sd    s0,  56(sp)
    sd    s1,  64(sp)
    sd    a0,  72(sp)
    sd    a1,  80(sp)
    sd    a2,  88(sp)
    sd    a3,  96(sp)
    sd    a4, 104(sp)
    sd    a5, 112(sp)
    sd    a6, 120(sp)
    sd    a7, 128(sp)
    sd    s2, 136(sp)
    sd    s3, 144(sp)
    sd    s4, 152(sp)
    sd    s5, 160(sp)
    sd    s6, 168(sp)
    sd    s7, 176(sp)
    sd    s8, 184(sp)
    sd    s9, 192(sp)
    sd    s10, 200(sp)
    sd    s11, 208(sp)
    sd    t3, 216(sp)
    sd    t4, 224(sp)
    sd    t5, 232(sp)
    sd    t6, 240(sp)

    # Call the Rust trap handler with a pointer to the TrapContext.
    mv    a0, sp
    call  trap_handler

    # Restore all registers except sp and t0 (handled last).
    ld    ra,   0(sp)
    ld    gp,  16(sp)
    ld    tp,  24(sp)
    ld    t1,  40(sp)
    ld    t2,  48(sp)
    ld    s0,  56(sp)
    ld    s1,  64(sp)
    ld    a0,  72(sp)
    ld    a1,  80(sp)
    ld    a2,  88(sp)
    ld    a3,  96(sp)
    ld    a4, 104(sp)
    ld    a5, 112(sp)
    ld    a6, 120(sp)
    ld    a7, 128(sp)
    ld    s2, 136(sp)
    ld    s3, 144(sp)
    ld    s4, 152(sp)
    ld    s5, 160(sp)
    ld    s6, 168(sp)
    ld    s7, 176(sp)
    ld    s8, 184(sp)
    ld    s9, 192(sp)
    ld    s10, 200(sp)
    ld    s11, 208(sp)
    ld    t3, 216(sp)
    ld    t4, 224(sp)
    ld    t5, 232(sp)
    ld    t6, 240(sp)

    # Restore sscratch to point back to trap_stack_top (virtual address,
    # because the MMU is on by the time any trap is handled in kmain).
    la    t0, trap_stack_top
    csrw  sscratch, t0

    # Restore t0 and the interrupted sp last.
    ld    t0,  32(sp)
    ld    sp,   8(sp)

    sret
