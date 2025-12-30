# BedRock Programming Language
BedRock is a minimalist, low-level, statically-typed systems programming language designed to run on Bare Metal (x86). It compiles directly to machine code, enabling the development of self-booting kernels and low-level system software without any external dependencies or an underlying operating system.
Technical Specifications
Data Types
 * u8: 8-bit unsigned integer.
 * u16: 16-bit unsigned integer.
 * u32: 32-bit unsigned integer.
 * void: Indicates no return value.
 * *T: Pointer to a specific type.
# Language Keywords and Built-in Functions
 * fn: Declares a function.
 * let: Declares a global or local variable.
 * loop: Initiates an infinite loop block.
 * asm: Allows embedding raw assembly instructions.
 * volatile: Prevents compiler optimizations on memory-mapped hardware.
 * clear(): Built-in command to clear the VGA text-mode screen.
 * print(string, color): Prints text to the screen using BIOS interrupts.
 * newline(): Moves the cursor to the next line.
# The Boot Process and Image Creation
To execute a BedRock kernel on physical hardware or an emulator, the compiled binary must be integrated with a bootloader and aligned to the hardware's sector requirements.
1. The Bootsector (boot.asm)
The system requires a standard 512-byte bootsector to load the kernel from the disk into memory and transfer control to the BedRock entry point.
[bits 16]
[org 0x7c00]

mov [BOOT_DRIVE], dl
mov bp, 0x9000
mov sp, bp

call load_kernel
jmp 0x1000 ; Jump to the BedRock kernel memory location

%include "disk.asm"

times 510-($-$$) db 0
dw 0xAA55

2. Manual Sector Padding and Alignment
BIOS reads disk data in fixed 512-byte sectors. If the compiled kernel does not perfectly fill a sector, it must be padded with null bytes to prevent hardware errors or undefined behavior.
Calculation Logic:
 * Compile the kernel to get kernel.bin. [ type in CMD with admin access insdie kernel.br directory 'bedrockco.exe kernel.br --format bin' ]
 * Determine the size of kernel.bin in bytes (represented as X).
 * Calculate the required padding: 512 - X = Padding Size.
Creating the Padding File (Windows CMD):
Use the fsutil tool to generate a file containing only null bytes:
[fsutil file createnew padding.bin [Padding Size] ]

Build and Execution Workflow
Follow these steps to build and boot the BedRock OS:
Step 1: Compile the Source Code
Translate your .br file into a raw binary using the BedRock compiler:
bedrockco.exe kernel.br --format bin

Step 2: Prepare the Bootloader
Assemble the bootsector using NASM:
nasm -f bin boot.asm -o boot.bin

Step 3: Align and Stitch the Image
Merge the components into a single bootable disk image (.img) using the binary copy command:
copy /b boot.bin + kernel.bin + padding.bin kernel.img

Step 4: Emulation
Run the final image in QEMU to verify the execution:
qemu-system-i386 -drive format=raw,file=kernel.img

Compiler Architecture
The BedRock compiler is built with three main modules:
 * Lexical Analyzer (Lexer): Converts source text into tokens.
 * Syntax Analyzer (Parser): Constructs an Abstract Syntax Tree (AST) based on language grammar.
 * Code Generator (CodeGen): Produces x86 machine code and manages memory offsets for local and global symbols.
