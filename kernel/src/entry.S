    .section .text.entry, "ax"
    .globl _start
    .type _start, @function
    .align 2

_start:
    # Relaxation could turn this into a load through an uninitialized gp.
    .option push
    .option norelax
    la gp, __global_pointer$
    .option pop

    # OpenSBI provides the hart ID in a0 and the device-tree address in a1.
    # Leave both registers untouched while establishing the bootstrap stack.
    la sp, __boot_stack_top
    andi sp, sp, -16
    mv s0, zero

    # The linker guarantees that both boundaries are 16-byte aligned, so
    # clearing one machine word at a time cannot run past the section.
    la t0, __bss_start
    la t1, __bss_end
.Lclear_bss:
    bgeu t0, t1, .Lbss_cleared
    sd zero, 0(t0)
    addi t0, t0, 8
    j .Lclear_bss
.Lbss_cleared:

    call rust_main

    # rust_main is non-returning; park the hart if that contract is violated.
.Lhalt:
    wfi
    j .Lhalt

    .size _start, . - _start
