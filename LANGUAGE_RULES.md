# 🔥 BedRock Language - Complete Rules & Guidelines

> **The authoritative reference for BedRock language design, semantics, and compiler behavior**

---

## GOLDEN RULES (Must Never Break)

### Rule 1: No Hidden Abstractions
Every construct maps to **exactly one sequence of MIPS32 instructions**. No runtime, no garbage collector, no virtual dispatch.

**Consequence**: You can always predict the exact machine code.

### Rule 2: Direct Hardware Access
`poke`, `peek`, `inb`, `outb` are first-class citizens. No OS abstraction layer.

**Consequence**: You own the metal. You control everything.

### Rule 3: Type Safety by Default
All types checked statically. Unsafe requires `unsafe` keyword.

**Consequence**: Most errors caught at compile time.

### Rule 4: Zero-Cost Abstractions
Traits, generics, higher-order functions cost nothing at runtime.

**Consequence**: Abstraction is free.

---

## SEMANTIC RULES

### Variable & Constant Rules

**Rule V1: Let Variables Are Mutable by Default at Block Scope**
```bedrock
let x = 5;           // x is immutable
let mut y = 5;       // y is mutable
y = 10;              // ✓ OK
x = 10;              // ✗ Error: cannot assign to immutable variable
```

**Rule V2: Constants Are Compile-Time Only**
```bedrock
const MAX: u32 = 256;           // ✓ Compile-time constant
const ARRAY: [u32; 3] = [1, 2, 3];  // ✓ OK
const COMPUTED: u32 = MAX + 1;  // ✓ OK (evaluated at compile time)

let computed = MAX + get_value(); // ✓ OK (evaluated at runtime)
```

**Rule V3: Static Variables Are Global and Mutable**
```bedrock
static mut GLOBAL_COUNTER: u32 = 0;
static CONST_DATA: [u32; 256] = [0; 256];

GLOBAL_COUNTER += 1;  // ✓ OK
CONST_DATA[0] = 42;   // ✓ OK (can mutate through reference)
```

**Rule V4: Uninitialized Variables Must Be Initialized Before Use**
```bedrock
let x: u32;
x = 5;       // ✓ Must assign before use
print(x);    // ✓ OK

let y: u32;
print(y);    // ✗ Error: used uninitialized
```

### Type Rules

**Rule T1: Type Inference Is Mandatory Where Unambiguous**
```bedrock
let x = 42;              // Inferred: u32
let y = 3.14;            // Inferred: f32
let z: u32 = 42;        // Explicit (unnecessary but allowed)
let w = get_value();    // Inferred from function return type

let ambiguous = [];     // ✗ Error: type cannot be inferred for empty array
```

**Rule T2: No Implicit Type Conversion**
```bedrock
let a: u32 = 42;
let b: u8 = a;          // ✗ Error: cannot implicitly convert u32 to u8
let b: u8 = a as u8;    // ✓ OK: explicit cast

let c: i32 = -5;
let d: u32 = c as u32;  // ✓ OK: explicit cast (truncates)
```

**Rule T3: Type Bounds Are Checked at Compile Time**
```bedrock
let a: u8 = 256;        // ✗ Error: value out of range for u8
let a: u8 = 255;        // ✓ OK: within range
```

**Rule T4: Generic Types Must Have Explicit Bounds**
```bedrock
fn max<T>(a: T, b: T) -> T { ... }  // ✗ Error: T is unbounded
fn max<T: Ord>(a: T, b: T) -> T { ... }  // ✓ OK: T must implement Ord
```

### Scope Rules

**Rule S1: Scope Hierarchy**
1. Global (module) scope
2. Function scope
3. Block scope
4. Inner block scope (nested)

**Rule S2: Shadowing Is Allowed**
```bedrock
let x = 5;
{
    let x = 10;      // ✓ Shadows outer x
    print(x);        // Prints 10
}
print(x);            // Prints 5
```

**Rule S3: Variables Don't Escape Their Scope**
```bedrock
let ptr: *u32;
{
    let local = 42;
    ptr = &local;    // ✗ Error: local doesn't live long enough
}
// ptr is dangling!
```

### Function Rules

**Rule F1: All Parameters Must Be Type-Annotated**
```bedrock
fn add(a: u32, b: u32) -> u32 { a + b }  // ✓ OK
fn add(a, b) -> u32 { a + b }            // ✗ Error: parameter types required
```

**Rule F2: Return Type Must Be Explicit (Unless Unit)**
```bedrock
fn get_value() -> u32 { 42 }    // ✓ OK
fn get_value() { 42 }           // ✓ OK (infers -> ())
fn side_effect() { }            // ✓ OK (implicit unit return)
```

**Rule F3: Return Without Value Returns Unit**
```bedrock
fn foo() -> () {
    return;          // ✓ OK: returns ()
}

fn bar() -> u32 {
    return;          // ✗ Error: expected u32, found ()
}
```

**Rule F4: Last Expression Is Implicit Return (If No Semicolon)**
```bedrock
fn add(a: u32, b: u32) -> u32 {
    a + b            // ✓ Implicit return
}

fn add(a: u32, b: u32) -> u32 {
    a + b;           // ✗ Error: returns () not u32
}
```

**Rule F5: Function Calls Require All Arguments**
```bedrock
fn add(a: u32, b: u32) -> u32 { a + b }

add(5, 3);           // ✓ OK
add(5);              // ✗ Error: missing argument
add(5, 3, 1);        // ✗ Error: too many arguments
```

### Array Rules

**Rule A1: Array Length Is Fixed at Compile Time**
```bedrock
let arr: [u32; 256] = [0; 256];  // ✓ OK
let size = 256;
let arr: [u32; size] = [0; size];  // ✗ Error: size must be const expression
```

**Rule A2: Array Index Must Be Compile-Time or Runtime u32**
```bedrock
let arr = [1, 2, 3, 4, 5];
arr[0];              // ✓ OK
arr[2];              // ✓ OK
let i = 2;
arr[i];              // ✓ OK: runtime index
```

**Rule A3: Array Index Out of Bounds Is Undefined**
```bedrock
let arr = [1, 2, 3];
arr[3];              // ✗ Undefined behavior at runtime
arr[1000000];        // ✗ Undefined behavior at runtime
```

### Memory Rules

**Rule M1: Pointers Are Raw (No Automatic Null Checking)**
```bedrock
let ptr: *u32 = 0xB8000000 as *u32;
let value = *ptr;    // ✓ OK: no null check
                     // May crash if address invalid
```

**Rule M2: Reference Lifetime Must Be Valid**
```bedrock
let ptr: &u32;
{
    let local = 42;
    ptr = &local;    // ✗ Error: reference to local
}
// ptr is dangling!
```

**Rule M3: mutable References Are Exclusive**
```bedrock
let mut x = 42;
let r1 = &mut x;
let r2 = &mut x;     // ✗ Error: cannot borrow x as mutable more than once
*r1 = 10;
*r2 = 20;
```

### Ownership Rules

**Rule O1: Moving Ownership**
```bedrock
let owned = allocate(256);
use(owned);          // owned moved into use()
print(owned);        // ✗ Error: owned was moved
```

**Rule O2: Borrowing Preserves Ownership**
```bedrock
let owned = allocate(256);
use_ref(&owned);     // owned borrowed
print(owned);        // ✓ OK: owned still valid
```

---

## OPERATOR RULES

### Precedence (Highest to Lowest)

```
1. :: (scope resolution)
2. . [] () (member access, indexing, call)
3. ! - * & (unary operators)
4. as (type cast)
5. * / % (multiplicative)
6. + - (additive)
7. << >> (shifts)
8. & (bitwise AND)
9. ^ (bitwise XOR)
10. | (bitwise OR)
11. == != < > <= >= (comparison)
12. && (logical AND - short circuit)
13. || (logical OR - short circuit)
14. .. ..= (range)
15. = += -= *= /= (assignment)
```

### Associativity

All operators are **left-associative** except assignment (right-associative):

```bedrock
a + b + c           // (a + b) + c
a = b = c = 5       // a = (b = (c = 5))
```

### Short-Circuit Evaluation

```bedrock
a && b              // b not evaluated if a is false
a || b              // b not evaluated if a is true
```

---

## CONTROL FLOW RULES

### If Statements

**Rule C1: If Condition Must Be Boolean**
```bedrock
if true { }          // ✓ OK
if 1 { }             // ✗ Error: expected bool, found u32
if x > 0 { }         // ✓ OK: comparison returns bool
```

**Rule C2: If Can Be Expression**
```bedrock
let max = if a > b { a } else { b };
let result = if x > 0 {
    10
} else {
    20
};
```

### Loop Statements

**Rule C3: Loop Is Infinite (Must Break or Return)**
```bedrock
loop {
    // Never exits unless break or return
}
```

**Rule C4: Break Without Label Exits Innermost Loop**
```bedrock
loop {
    loop {
        break;       // Breaks inner loop only
    }
    // Continues outer loop
}
```

**Rule C5: Continue Skips to Next Iteration**
```bedrock
while condition {
    if skip { continue; }
    process();
}
```

---

## TRAIT & IMPL RULES

### Trait Rules

**Rule Tr1: Trait Methods Must Have Receiver (self, &self, or &mut self)**
```bedrock
trait Device {
    fn read(self) -> u32;        // ✓ OK: consumes self
    fn write(&mut self, v: u32); // ✓ OK: borrows mutably
    fn status(&self) -> u32;     // ✓ OK: borrows immutably
    fn reset() -> Self;          // ✓ OK: static method (no receiver)
}
```

**Rule Tr2: Impl Block Can Have Generic Bounds**
```bedrock
impl<T: Clone> MyType<T> {
    // T must implement Clone
}
```

**Rule Tr3: Trait Objects Use Dynamic Dispatch**
```bedrock
let device: &dyn Device = &uart;  // Dynamic dispatch
device.read();                     // Resolved at runtime
```

---

## HARDWARE ACCESS RULES

### poke / peek

**Rule H1: poke Writes 32-Bit Value**
```bedrock
poke(address: u32, value: u32)

poke(0xB8000, 72);   // Write 'H' to VGA
```

**Rule H2: peek Reads 32-Bit Value**
```bedrock
peek(address: u32) -> u32

let status = peek(GPIO_STATUS);  // Read GPIO status
```

**Rule H3: Address Must Be Valid**
```bedrock
poke(0xB8000, 42);   // ✓ Valid VGA memory
poke(0x00000, 42);   // ✗ Likely invalid (kernel memory)
```

### Interrupts

**Rule H4: Interrupts Require Explicit Enable**
```bedrock
enable_interrupts();     // Required before interrupts work
disable_interrupts();    // Disable in critical section
```

---

## COMPILATION RULES

### Optimization Levels

- **Level 0**: No optimization (debug)
- **Level 1**: Constant folding, dead code elimination
- **Level 2**: Peephole optimization, instruction scheduling
- **Level 3**: Register allocation optimization, loop unrolling

### Inline Assembly

**Rule As1: Inline Assembly Must Be String Literal**
```bedrock
asm!("nop");                 // ✓ OK
asm!(instruction_str);       // ✗ Error: must be literal
```

**Rule As2: Inline Assembly Is Unverified**
```bedrock
asm!("nop");                 // ✓ Emitted as-is
asm!("invalid mips");        // ✗ Silently assembles to 0x00000000
```

---

## ERROR HANDLING RULES

### Result Type

**Rule E1: Result Must Be Handled**
```bedrock
fn read_file() -> Result<Vec<u8>, Error> { }

read_file()?;            // ✓ Propagate error
let data = read_file()?; // ✓ Unwrap or propagate

read_file();             // ⚠ Warning: Result not used
```

**Rule E2: Unwrap Panics on Error**
```bedrock
let data = read_file().unwrap();  // Panics if Err
```

---

## DOCUMENTATION RULES

### Comments

**Rule D1: Comments Can Be Multi-Line**
```bedrock
// Single-line comment

/* Multi-line
   comment */

/* Nested /* comments */ are supported */
```

**Rule D2: Doc Comments Generate Docs**
```bedrock
/// This function adds two numbers
pub fn add(a: u32, b: u32) -> u32 { a + b }

//! This module handles UART operations
```

---

## SPECIAL SEMANTICS

### Semicolon Behavior

**The Semicolon Rule**: Semicolon **consumes the return value**

```bedrock
let x = { 5 };        // x = 5 (no semicolon)
let x = { 5; };       // x = () (semicolon)

fn f() -> u32 {
    5                 // Returns 5
}

fn f() -> () {
    5;                // Returns ()
}
```

### Block Expressions

```bedrock
let max = {
    let a = 5;
    let b = 10;
    if a > b { a } else { b }  // Returns the maximum
};
```

### Pattern Matching

**Rule P1: Match Must Be Exhaustive**
```bedrock
match option {
    Some(x) => process(x),
    None => handle_none(),
}

match option {
    Some(x) => process(x),
    // ✗ Error: missing case for None
}
```

---

## PERFORMANCE RULES

### Inlining

```bedrock
#[inline]
fn fast_operation() { }  // Inlined at every call site

#[inline(always)]
fn must_inline() { }     // Aggressive inlining

#[inline(never)]
fn never_inline() { }    // Never inlined
```

### Register Allocation

BedRock uses greedy register allocation:
- 8 temporary registers available
- Spills to stack if exhausted
- No register pressure optimization (yet)

---

## LIMITS & CONSTRAINTS

| Constraint | Limit |
|------------|-------|
| Function parameters | Unlimited (4 in registers, rest on stack) |
| Local variables | Stack size dependent |
| Array elements | 2^32 maximum |
| String length | Stack/heap dependent |
| Module nesting | Unlimited |
| Generic parameters | Unlimited |
| Trait bounds | Unlimited |
| Compiler output size | ~1-5 MB (typical program) |

---

## FORBIDDEN PATTERNS

```bedrock
// ✗ Cannot use uninitialized variable
let x: u32;
print(x);

// ✗ Cannot shadow with different type in same scope
let x: u32 = 5;
let x: i32 = 5;    // ✗ (not allowed in same scope)

// ✗ Cannot pass mutable ref twice
let mut x = 5;
let r1 = &mut x;
let r2 = &mut x;   // ✗ Exclusive borrow violated

// ✗ Cannot use moved value
let owned = allocate(256);
use(owned);
use(owned);        // ✗ Already moved

// ✗ Cannot break outside loop
if x > 0 {
    break;         // ✗ Not in loop
}

// ✗ Cannot return from loop
loop {
    return x;      // ✓ OK: loop can exit via return
}

// ✗ Recursive without return type
fn foo() { foo(); } // ✗ Must have explicit return type

// ✗ Type mismatch in array
let arr = [1, "two", 3]; // ✗ Mixed types
```

---

## BEST PRACTICES

### 1. Always Specify Return Types for Public Functions
```bedrock
pub fn get_status() -> u32 { }  // ✓ Good
pub fn get_status() { }         // ⚠ Unclear
```

### 2. Use Explicit Types for Complex Expressions
```bedrock
let complex: Result<u32, Error> = operation();  // ✓ Clear
let complex = operation();                       // ⚠ Harder to follow
```

### 3. Prefer Pattern Matching Over If-Else
```bedrock
match result {
    Ok(v) => process(v),
    Err(e) => handle_error(e),
}  // ✓ Exhaustive

if result.is_ok() {  // ⚠ Not exhaustive without else
    process(result.unwrap());
}
```

### 4. Use Named Constants for Magic Numbers
```bedrock
const UART_BASE: u32 = 0xB8000000;
poke(UART_BASE, 42);  // ✓ Clear intent

poke(0xB8000000, 42); // ⚠ Magic number
```

### 5. Document Unsafe Code
```bedrock
// ✓ Good documentation
unsafe {
    // SAFETY: we verified that ptr is valid and properly aligned
    *ptr = 42;
}

unsafe { *ptr = 42; }  // ⚠ Why is this safe?
```

---

## COMPILER PHASES GUARANTEES

1. **Lexical**: All valid tokens recognized
2. **Parse**: All valid BedRock syntax accepted
3. **Type Check**: All type mismatches caught
4. **Semantic**: All use-before-def caught
5. **IR**: All high-level semantics preserved
6. **Optimize**: Semantics preserved (optimization correctness)
7. **Codegen**: 1-to-1 mapping to MIPS32
8. **Link**: Valid ELF binary generated

---

**BedRock Language Rules & Guidelines - COMPLETE** ✅
