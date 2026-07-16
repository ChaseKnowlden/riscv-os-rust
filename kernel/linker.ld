OUTPUT_ARCH(riscv)
ENTRY(_start)

/* OpenSBI conventionally loads supervisor payloads at this address on QEMU virt. */
KERNEL_BASE = 0x80200000;
PAGE_SIZE = 4K;
BOOT_STACK_SIZE = 16K;

PHDRS
{
    text   PT_LOAD FLAGS(5); /* Read + execute. */
    rodata PT_LOAD FLAGS(4); /* Read only. */
    data   PT_LOAD FLAGS(6); /* Read + write. */
}

SECTIONS
{
    . = KERNEL_BASE;
    __kernel_start = .;

    . = ALIGN(PAGE_SIZE);
    __text_start = .;
    .text :
    {
        KEEP(*(.text.entry))
        *(.text .text.*)
    } :text
    __text_end = .;

    . = ALIGN(PAGE_SIZE);
    __rodata_start = .;
    .rodata :
    {
        *(.rodata .rodata.*)
        *(.srodata .srodata.*)
        *(.data.rel.ro .data.rel.ro.*)
    } :rodata
    __rodata_end = .;

    . = ALIGN(PAGE_SIZE);
    __data_start = .;
    .data :
    {
        *(.data .data.*)

        /* RISC-V small data is addressed relative to the global pointer. */
        . = ALIGN(8);
        PROVIDE(__global_pointer$ = . + 0x800);
        *(.sdata .sdata.*)

        *(.got .got.*)
    } :data
    __data_end = .;

    . = ALIGN(PAGE_SIZE);
    __bss_start = .;
    .bss (NOLOAD) :
    {
        *(.sbss .sbss.*)
        *(.bss .bss.*)
        *(COMMON)
        . = ALIGN(16);
    } :data
    __bss_end = .;

    /* Temporary stack used until per-hart kernel stacks are available. */
    . = ALIGN(PAGE_SIZE);
    __boot_stack_bottom = .;
    .boot_stack (NOLOAD) :
    {
        . += BOOT_STACK_SIZE;
        . = ALIGN(16);
    } :data
    __boot_stack_top = .;

    . = ALIGN(PAGE_SIZE);
    __kernel_end = .;

    /DISCARD/ :
    {
        *(.comment)
        *(.eh_frame .eh_frame.*)
        *(.note .note.*)
    }
}
