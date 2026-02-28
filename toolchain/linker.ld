ENTRY(kernel_main)

SECTIONS
{
    /* BIOS boot sector starts at 0x7C00, but for a kernel we usually load higher.
       Using 0x100000 (1MB) is standard for protected mode kernels. */
    . = 0x100000;

    .text : {
        *(.text.kernel_main) /* Ensure kernel_main is first if needed, or just standard .text */
        *(.text*)
    }

    .rodata : {
        *(.rodata*)
    }

    .data : {
        *(.data*)
    }

    .bss : {
        *(.bss*)
        *(COMMON)
    }

    /DISCARD/ : {
        *(.eh_frame)
        *(.note.gnu.build-id)
    }
}
