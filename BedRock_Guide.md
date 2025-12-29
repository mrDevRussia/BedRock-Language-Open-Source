# BedRock Language Guide

BedRock is a strict, zero-runtime, bare-metal programming language designed for Operating System development and high-performance systems programming. It provides low-level control with a syntax similar to Rust and C.

## Table of Keywords

| Keyword    | Description |
|------------|-------------|
| `fn`       | Defines a function. |
| `let`      | Declares a variable (global or local). |
| `volatile` | Marks a global variable as volatile (preventing compiler optimization), typically for MMIO. |
| `cast`     | Performs an explicit type conversion. |
| `struct`   | *Reserved for future use.* |
| `unsafe`   | Marks a block of code that performs unsafe operations (often required for system programming). |
| `loop`     | Defines an infinite loop. |
| `asm`      | Embeds inline assembly code. |

## Detailed Language Reference

### 1. Variables (`let`, `volatile`)
Variables are declared using the `let` keyword and must have an explicit type. Global variables can be marked as `volatile` if they represent hardware registers.

**Syntax:**
```rust
let variable_name: Type = value;
```

**Global MMIO Example:**
```rust
// Define a pointer to a VGA buffer or hardware register
#[address(0xB8000)]
volatile let vga_buffer: *u16;
```

### 2. Functions (`fn`)
Functions are the building blocks of BedRock programs. They must specify a return type (use `void` if none).

**Syntax:**
```rust
fn function_name(arg1: Type, arg2: Type) -> ReturnType {
    // Body
}
```

**Interrupt Handlers:**
You can mark functions as interrupt handlers using the `#[interrupt]` attribute.
```rust
#[interrupt]
fn keyboard_handler() -> void {
    // Handle IRQ
}
```

### 3. Type Casting (`cast`)
BedRock is strongly typed. Use `cast` to convert between types, such as integers to pointers.

**Syntax:**
```rust
cast<TargetType>(expression)
```

**Example:**
```rust
let ptr_val: u64 = 0xB8000;
let vga_ptr: *u16 = cast<*u16>(ptr_val);
```

### 4. Control Flow (`loop`)
The `loop` keyword creates an infinite loop, equivalent to `while(true)` in C.

**Example:**
```rust
loop {
    // This code runs forever
}
```

### 5. Unsafe Blocks (`unsafe`)
Operations that bypass safety checks or interact directly with hardware often live inside `unsafe` blocks.

**Example:**
```rust
unsafe {
    // Low-level operations
}
```

### 6. Inline Assembly (`asm`)
You can inject raw assembly instructions directly into your code using `asm`.

**Example:**
```rust
asm("hlt"); // Halt the CPU
```

## Data Types

| Type | Description |
|------|-------------|
| `u8` - `u64` | Unsigned integers (8, 16, 32, 64-bit) |
| `i8` - `i64` | Signed integers |
| `f32`, `f64` | Floating point numbers |
| `bool` | Boolean (`true`/`false`) |
| `void` | No return value |
| `*T` | Pointer to type T (e.g., `*u8`) |

## Attributes
BedRock uses attributes to control compiler behavior for specific items.

- `#[address(0x...)]`: Fixes a global variable to a specific memory address.
- `#[align(N)]`: Aligns the item to N bytes.
- `#[interrupt]`: Marks a function as an interrupt service routine.

## Basic Program Template (Hello World)

Since BedRock is a bare-metal language, a "Hello World" typically involves writing directly to video memory (like the VGA text buffer on x86).

```rust
// VGA Buffer pointer at memory address 0xB8000
#[address(0xB8000)]
volatile let vga_buffer: *u16;

fn main() -> void {
    // 'H' (0x48) with white-on-black color (0x0F) -> 0x0F48
    unsafe {
        *vga_buffer = 0x0F48;
        
        // Move to next character slot (2 bytes per char)
        let next_char: *u16 = cast<*u16>(cast<u64>(vga_buffer) + 2);
        
        // 'i' (0x69) -> 0x0F69
        *next_char = 0x0F69;
    }

    // Halt the CPU loop
    loop {
        asm("hlt");
    }
}
```
