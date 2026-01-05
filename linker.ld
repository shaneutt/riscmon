/* Linker script for riscmon (bare-metal, no_std) */

/* QEMU virt loads the kernel at this physical address. */
BASE_ADDRESS = 0x80000000;
OUTPUT_ARCH(riscv)
ENTRY(_start)

SECTIONS
{
    /* Set location counter to start of RAM */
    . = BASE_ADDRESS;

    /*
     * .text - executable code.
     * We force .text.entry first so the _start symbol lands right at BASE_ADDRESS.
     */
    .text : {
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
        *(.bss .bss.*)
    }
}
