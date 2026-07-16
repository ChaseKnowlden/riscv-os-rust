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

    call rust_main

    # rust_main is non-returning; park the hart if that contract is violated.
.Lhalt:
    wfi
    j .Lhalt

    .size _start, . - _start

