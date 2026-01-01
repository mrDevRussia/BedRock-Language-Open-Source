# BedRock systems programming language 


Functional Keywords & Usage
These are the core keywords currently supported by the BedRock compiler:
| Keyword | Description | Real-world OS Usage |
|---|---|---|
| fn | Defines a function. | fn kernel_main() -> void - The entry point of your OS. |
| let | Declares a variable or memory pointer. | let vga: *u8 = 0xB8000; - Targeting the screen buffer. |
| void | Specifies a function that returns nothing. | Used for procedures like clear() or kernel_main. |
| loop | Creates an infinite execution block. | Used at the end of the kernel to prevent the CPU from executing random memory. |
| asm | Inlines raw Assembly instructions. | asm("hlt"); - Putting the CPU in a halt state to save power. |
| u8 / u16 | Unsigned 8-bit and 16-bit integers. | u8 for ASCII characters; u16 for VGA words (character + attribute). |


Built-in Hardware Commands
BedRock includes specialized commands that map directly to BIOS Interrupts and Hardware I/O:
 * clear(): Resets the VGA text buffer, clearing the screen to black.
 * print("text", color): Invokes BIOS int 10h to render text. The color parameter accepts a u8 attribute (e.g., 14 for Yellow, 10 for Green).
 * newline(): Moves the hardware cursor to the beginning of the next line.



Memory Management (Pointers)
BedRock treats memory as a raw array of bytes. Using the * operator, you can perform Direct Memory Access (DMA):


    * // Writing '!' (ASCII 33) in Light Red (Color 12) directly to VGA memory
    fn apply_visual_fix() -> void {
     let char_ptr: *u8 = 0xB8000;
     let color_ptr: *u8 = 0xB8001; 
     char_ptr = 33;  // The '!' symbol
     color_ptr = 12; // The Light Red attribute
    }
  

Build & Deployment
1. Compile to Raw Binary
bedrockco.exe kernel.br --format bin

2. Generate Padding
Ensures the kernel fits perfectly into disk sectors:
fsutil file createnew padding.bin (kernel.bin file size - 512)

3. Create Disk Image
Concatenate the Bootloader, Kernel, and Padding:
copy /b boot.bin + kernel.bin + padding.bin BedRockOS.img

4. Boot in Emulator
