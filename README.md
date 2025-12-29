# BedRock Programming Language

BedRock is a strict, zero-runtime, bare-metal programming language designed specifically for Operating System development, firmware, and high-performance systems programming. It aims to provide low-level control similar to C and Assembly but with a more structured syntax and modern safety features where possible.

BedRock is designed to boot directly on x86_64 or ARM hardware without any underlying OS or C-standard library.

## Installation & Setup

### Prerequisites
To build the BedRock compiler, you need **Rust** installed on your system.

### Building the Compiler
1. Clone the repository.
2. Navigate to the `compiler` directory.
3. Build the compiler using Cargo:

```bash
cd compiler
cargo build --release
```

This will produce the `bedrockc.exe` executable in the `target/release/` directory.

## Compiler CLI Usage

The BedRock compiler (`bedrockc`) provides a simple command-line interface for compiling `.br` source files into raw binaries or assembly code.

### Usage Syntax
```bash
bedrockc <source_file> [options]
```

### Options

| Flag | Description |
|------|-------------|
| `--format bin` | Compile the source file into a raw binary (flat binary) suitable for kernels. |
| `--format asm` | Generate x86 assembly code (`.asm`) from the source file. |

### Examples

**Generate a raw binary:**
```bash
bedrockc kernel.br --format bin
```

**Generate assembly output:**
```bash
bedrockc kernel.br --format asm
```

## Syntax & Grammar Guide

BedRock uses a strongly-typed syntax with explicit keywords for system-level operations.

### Variables
Variables are declared using the `let` keyword. Type annotations are mandatory.

```rust
let number: u32 = 1234;
```

**Volatile Variables:**
For hardware registers or memory-mapped I/O that changes outside the program's control, use the `volatile` keyword.

```rust
volatile let status_reg: *u8;
```

### Functions
Functions are defined using the `fn` keyword. Return types must be specified (use `void` for no return).

```rust
fn add(a: u32, b: u32) -> u32 {
    let sum: u32 = a + b;
    // implicit return not yet supported, use assignment or expression
}
```

**Interrupt Handlers:**
Special functions can be marked as interrupt handlers.

```rust
#[interrupt]
fn keyboard_handler() -> void {
    // Handle interrupt
}
```

### Types
BedRock supports standard fixed-width integer types, floating-point types, boolean, and pointers.

| Type | Description |
|------|-------------|
| `u8`, `u16`, `u32`, `u64` | Unsigned Integers (8-bit to 64-bit) |
| `i8`, `i16`, `i32`, `i64` | Signed Integers (8-bit to 64-bit) |
| `f32`, `f64` | Floating Point Numbers |
| `bool` | Boolean (`true` / `false`) |
| `void` | Empty type (used for return types) |
| `*T` | Pointer to type `T` (e.g., `*u16`) |

### Attributes
Attributes provide metadata to the compiler for specific memory layout or behavior.

*   `#[address(0x...)]`: Fixes a global variable to a specific physical memory address.
*   `#[align(n)]`: Aligns the variable or function to an `n`-byte boundary.
*   `#[interrupt]`: Marks a function as an interrupt service routine.

### Blocks

**Unsafe Blocks:**
Operations that directly manipulate memory or hardware must be enclosed in an `unsafe` block.

```rust
unsafe {
    *video_memory = 0x0F41;
}
```

**Loop Blocks:**
Infinite loops are created using the `loop` keyword.

```rust
loop {
    // Run forever
}
```

### Inline Assembly
BedRock allows embedding raw assembly instructions using `asm`.

```rust
asm("hlt");
```

## Technical Specifications

### Abstract Syntax Tree (AST)
The BedRock compiler processes source code into an Abstract Syntax Tree (AST). The structure is defined as follows:

*   **Program**: A collection of `TopLevelItem`s.
*   **TopLevelItem**: Can be a `GlobalVariable` or a `Function`.
    *   **GlobalVariable**: Contains name, type, volatility, and attributes.
    *   **Function**: Contains name, return type, attributes, and a body (block of statements).
*   **Statement**: Executable units inside functions.
    *   `Let`: Variable declaration.
    *   `UnsafeBlock`: Group of unsafe operations.
    *   `LoopBlock`: Infinite loop structure.
    *   `Assignment`: Assigning values to variables or memory locations.
    *   `ExpressionStmt`: An expression evaluated for its side effects.
*   **Expression**: Values and computations.
    *   Includes Integers, Identifiers, Casts (`cast<T>(v)`), Dereferences (`*ptr`), Binary Operations, Function Calls, and Inline Assembly.

## Code Example

Below is a fully documented example of a minimal kernel written in BedRock (`kernel.br`). This example demonstrates writing to the VGA video memory buffer.

```rust
// Define a pointer to the VGA text buffer at physical address 0xB8000
// 'volatile' ensures the compiler doesn't optimize away reads/writes
#[address(0xB8000)]
volatile let VIDEO_MEM: *u16;

// Define an interrupt handler for the keyboard (example)
#[interrupt]
fn keyboard_handler() -> void {
    // Read from I/O port 0x60 (keyboard data)
    let code: u8 = inb(cast<u16>(0x60));
}

// The main kernel entry point
fn kernel_main() -> void {
    // 0x0F00 = White text on Black background
    let color: u16 = cast<u16>(0x0F00);
    
    // 0x42 = ASCII character 'B'
    let letter: u16 = cast<u16>(0x42); 

    // Unsafe block required for raw pointer dereferencing
    unsafe {
        // Write 'B' with white color to the first VGA cell
        *VIDEO_MEM = letter | color;
    }

    // Infinite loop to halt the CPU
    loop { 
        asm("hlt"); 
    }
}
```
