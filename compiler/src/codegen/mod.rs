use crate::ast::{Statement, Expression};
use std::collections::{HashMap, HashSet, VecDeque};

pub struct Codegen { 
    code: Vec<u32>, 
    symbols: HashMap<String, u32>,      // Variable name -> Memory address
    root_symbols: HashMap<String, u32>, // Root symbols (constants) -> Direct values
    array_names: HashSet<String>,       // Names of arrays (for parameter passing)
    functions: HashMap<String, usize>,  // Function name -> Code position
    loop_stack: Vec<LoopContext>,       // Stack for nested loops (while/loop)
    current_params: HashMap<String, u32>, // Function parameters -> stack offset
    reg_pool: VecDeque<u32>,
    next_addr: u32,                     // Next available memory address for allocation
    static_data: Vec<(u32, Vec<u8>)>,   // (address, bytes) for static data section
}

struct LoopContext {
    start_addr: usize,      // Code position of loop start
    break_patches: Vec<usize>, // Positions that need patching for breaks
}

impl Codegen {
    pub fn new() -> Self {
        let mut pool = VecDeque::new();
        // $t0-$t7 (registers 8-15) available for general use
        for i in 8..=15 { pool.push_back(i); } 
        Codegen { 
            code: Vec::new(), 
            symbols: HashMap::new(),
            root_symbols: HashMap::new(),
            array_names: HashSet::new(),
            functions: HashMap::new(), 
            loop_stack: Vec::new(), 
            current_params: HashMap::new(), 
            reg_pool: pool, 
            next_addr: 0x80001000,  // Start data section at 0x80001000
            static_data: Vec::new(),
        }
    }

    pub fn compile(&mut self, stmts: &[Statement]) -> Vec<u8> {
        // ===== PHASE 1: BOOTSTRAP =====
        // Setup processor state and stack pointer
        self.emit(0x40806000);      // mtc0 $zero, $12 (disable interrupts)
        self.emit(0x3C1D8010);      // lui $sp, 0x8010 (set stack to 0x80100000)
        self.emit(0x37BD0000);      // ori $sp, $sp, 0x0000
        
        // Reserve space for jump to global code (will be patched)
        let global_jump_patch = self.code.len();
        self.emit(0x08000000);      // j <will_be_patched>
        self.emit(0x00000000);      // nop (delay slot)

        // ===== PHASE 2: COLLECT ROOTS =====
        // Register all root symbols (address definitions) - these are CONSTANTS
        for s in stmts {
            if let Statement::Root(name, expr) = s {
                if let Expression::Number(val) = expr {
                    self.root_symbols.insert(name.clone(), *val as u32);
                }
            }
        }

        // ===== PHASE 3: ALLOCATE STATIC DATA =====
        // Pre-allocate addresses for arrays and strings WITHOUT generating code yet
        for s in stmts {
            match s {
                Statement::ArrayDefine(name, vals) => {
                    let addr = self.next_addr;
                    self.symbols.insert(name.clone(), addr);
                    self.array_names.insert(name.clone());  // Track array names
                    
                    // Store data in static section
                    let mut bytes = Vec::new();
                    for v in vals {
                        bytes.extend_from_slice(&(*v as u32).to_be_bytes());
                    }
                    self.static_data.push((addr, bytes));
                    
                    self.next_addr += (vals.len() * 4) as u32;
                }
                Statement::StringDefine(name, s_val) => {
                    let addr = self.next_addr;
                    self.symbols.insert(name.clone(), addr);
                    self.array_names.insert(name.clone());  // Track array names
                    
                    // Store string bytes
                    let bytes = s_val.as_bytes().to_vec();
                    self.static_data.push((addr, bytes));
                    
                    // Align to 4 bytes
                    self.next_addr += s_val.len() as u32;
                    self.next_addr = (self.next_addr + 3) & !3;
                }
                _ => {}
            }
        }

        // ===== PHASE 4: GENERATE FUNCTION CODE =====
        // Functions are generated first so they have predictable addresses
        for s in stmts {
            if let Statement::FunctionDefine(_, _, _) = s {
                self.generate_stmt(s);
            }
        }

        // ===== PHASE 4.5: INITIALIZE STATIC DATA =====
        // Patch the initial jump to point to static data initialization
        let init_start = self.code.len();
        self.code[global_jump_patch] = 0x08000000 | ((init_start as u32) & 0x03FFFFFF);
        
        // Generate initialization code for arrays and strings
        // Clone to avoid borrow checker issues
        let static_data_copy = self.static_data.clone();
        for (addr, bytes) in static_data_copy.iter() {
            let mut offset = 0;
            for chunk in bytes.chunks(4) {
                // Build 32-bit value from bytes (Big-Endian)
                let mut value = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    value |= (byte as u32) << ((3 - i) * 8);
                }
                
                // Load value into register
                let val_reg = self.alloc_reg();
                self.emit_li(val_reg, value);
                
                // Load target address
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, addr + offset);
                
                // sw val_reg, 0(addr_reg)
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16));
                
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
                
                offset += chunk.len() as u32;
            }
        }

        // ===== PHASE 5: GENERATE GLOBAL CODE =====
        // Continue to global code (no jump needed, execution flows naturally)
        
        // Generate code for statements outside functions
        for s in stmts {
            if !matches!(s, Statement::Root(_, _) | Statement::FunctionDefine(_, _, _) 
                           | Statement::ArrayDefine(_, _) | Statement::StringDefine(_, _)) {
                self.generate_stmt(s);
            }
        }

        // ===== PHASE 6: HALT =====
        // Infinite loop at end of global code
        let halt_pos = self.code.len() as u32;
        self.emit(0x08000000 | (halt_pos & 0x03FFFFFF)); // j halt
        self.emit(0x00000000); // nop (delay slot)

        // ===== PHASE 7: GENERATE BINARY =====
        let mut binary = Vec::new();
        
        // Write code section (Big-endian as per MIPS standard)
        for &instr in &self.code {
            binary.extend_from_slice(&instr.to_be_bytes());
        }
        
        // Write static data section (if needed in future)
        // For now, static data is written at runtime by initialization code
        
        binary
    }

    fn generate_stmt(&mut self, s: &Statement) {
        match s {
            // ===== MEMORY OPERATIONS =====
            Statement::Poke(addr_expr, val_expr) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                // sb val_reg, 0(addr_reg) - store byte
                self.emit(0xA0000000 | (addr_reg << 21) | (val_reg << 16));
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }
            
            Statement::Outb(addr_expr, val_expr) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                // sb val_reg, 0(addr_reg) - store byte to I/O port
                self.emit(0xA0000000 | (addr_reg << 21) | (val_reg << 16));
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }

            // ===== STATIC DATA (already allocated, skip) =====
            Statement::ArrayDefine(_, _) | Statement::StringDefine(_, _) => {
                // Already handled in Phase 3
            }

            // ===== FUNCTIONS =====
            Statement::FunctionDefine(name, params, body) => {
                // Record function entry point
                self.functions.insert(name.clone(), self.code.len());
                
                // Function prologue
                self.emit(0x27BDFFE0);  // addiu $sp, $sp, -32 (allocate stack frame)
                self.emit(0xAFBF001C);  // sw $ra, 28($sp) (save return address)
                
                // Map parameters to stack offsets
                // Parameters were pushed BEFORE stack frame allocation
                // So they're now at $sp + 32 + (param_index * 4)
                for (i, param_name) in params.iter().enumerate() {
                    let offset = 32 + (i * 4);  // Add 32 to account for stack frame
                    self.current_params.insert(param_name.clone(), offset as u32);
                }
                
                // Generate function body
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                
                // Function epilogue (if no explicit return was generated)
                self.emit(0x8FBF001C);  // lw $ra, 28($sp)
                self.emit(0x27BD0020);  // addiu $sp, $sp, 32
                self.emit(0x03E00008);  // jr $ra
                self.emit(0x00000000);  // nop (delay slot)
                
                self.current_params.clear();
            }

            Statement::Call(func_name, args) => {
                // Push arguments to stack
                for (i, arg) in args.iter().enumerate() {
                    let arg_reg = self.alloc_reg();
                    self.gen_expr(arg, arg_reg);
                    // sw arg_reg, i*4($sp)
                    self.emit(0xAFA00000 | (arg_reg << 16) | ((i * 4) as u32 & 0xFFFF));
                    self.free_reg(arg_reg);
                }
                
                // JAL to function
                if let Some(&func_addr) = self.functions.get(func_name) {
                    self.emit(0x0C000000 | ((func_addr as u32) & 0x03FFFFFF));
                    self.emit(0x00000000); // nop (delay slot)
                }
            }

            Statement::Return => {
                self.emit(0x8FBF001C);  // lw $ra, 28($sp)
                self.emit(0x27BD0020);  // addiu $sp, $sp, 32
                self.emit(0x03E00008);  // jr $ra
                self.emit(0x00000000);  // nop
            }

            // ===== VARIABLES =====
            Statement::Let(name, value) | Statement::Assignment(name, value) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(value, val_reg);
                
                // Allocate memory address if not exists
                let addr = *self.symbols.entry(name.clone()).or_insert_with(|| {
                    let a = self.next_addr;
                    self.next_addr += 4;
                    a
                });
                
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, addr);
                
                // sw val_reg, 0(addr_reg)
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16));
                
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }

            // ===== CONTROL FLOW =====
            Statement::Loop(body) => {
                let start_pc = self.code.len();
                
                // Push new loop context
                self.loop_stack.push(LoopContext {
                    start_addr: start_pc,
                    break_patches: Vec::new(),
                });
                
                // Generate loop body
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                
                // Jump back to start
                self.emit(0x08000000 | ((start_pc as u32) & 0x03FFFFFF));
                self.emit(0x00000000); // nop
                
                // Patch all break statements
                let end_pc = self.code.len();
                if let Some(ctx) = self.loop_stack.pop() {
                    for pos in ctx.break_patches {
                        self.code[pos] = 0x08000000 | ((end_pc as u32) & 0x03FFFFFF);
                    }
                }
            }

            Statement::While(condition, body) => {
                let start_pc = self.code.len();
                
                // Push new loop context
                self.loop_stack.push(LoopContext {
                    start_addr: start_pc,
                    break_patches: Vec::new(),
                });
                
                // Evaluate condition
                let cond_reg = self.alloc_reg();
                self.gen_expr(condition, cond_reg);
                
                // Branch if zero (condition false) - will be patched
                let branch_patch = self.code.len();
                self.emit(0x10000000 | (cond_reg << 21)); // beq $cond_reg, $zero, <end>
                self.emit(0x00000000); // nop (delay slot)
                
                self.free_reg(cond_reg);
                
                // Generate loop body
                for stmt in body {
                    self.generate_stmt(stmt);
                }
                
                // Jump back to condition check
                self.emit(0x08000000 | ((start_pc as u32) & 0x03FFFFFF));
                self.emit(0x00000000); // nop
                
                // Patch branch target
                let end_pc = self.code.len();
                let offset = ((end_pc as i32) - (branch_patch as i32) - 1) as u16;
                self.code[branch_patch] = 0x10000000 | (cond_reg << 21) | (offset as u32);
                
                // Patch all break statements
                if let Some(ctx) = self.loop_stack.pop() {
                    for pos in ctx.break_patches {
                        self.code[pos] = 0x08000000 | ((end_pc as u32) & 0x03FFFFFF);
                    }
                }
            }

            Statement::If(condition, then_body, else_body) => {
                // Evaluate condition
                let cond_reg = self.alloc_reg();
                self.gen_expr(condition, cond_reg);
                
                // Branch if zero (condition false)
                let else_branch = self.code.len();
                self.emit(0x10000000 | (cond_reg << 21)); // beq $cond_reg, $zero, <else>
                self.emit(0x00000000); // nop
                
                self.free_reg(cond_reg);
                
                // Generate then block
                for stmt in then_body {
                    self.generate_stmt(stmt);
                }
                
                // Jump over else block
                let end_jump = self.code.len();
                self.emit(0x08000000); // j <end>
                self.emit(0x00000000); // nop
                
                // Patch else branch
                let else_pc = self.code.len();
                let else_offset = ((else_pc as i32) - (else_branch as i32) - 1) as u16;
                self.code[else_branch] = 0x10000000 | (cond_reg << 21) | (else_offset as u32);
                
                // Generate else block (if exists)
                if let Some(else_stmts) = else_body {
                    for stmt in else_stmts {
                        self.generate_stmt(stmt);
                    }
                }
                
                // Patch end jump
                let end_pc = self.code.len();
                self.code[end_jump] = 0x08000000 | ((end_pc as u32) & 0x03FFFFFF);
            }

            Statement::Break => {
                // Jump to end of loop (will be patched)
                let pos = self.code.len();
                self.emit(0x08000000); // j <end_of_loop>
                self.emit(0x00000000); // nop
                
                // Record position for patching
                if let Some(ctx) = self.loop_stack.last_mut() {
                    ctx.break_patches.push(pos);
                }
            }

            Statement::Asm(_) => {
                // Raw assembly not implemented yet
            }

            Statement::Root(_, _) => {
                // Root statements are handled in Phase 2 of compile()
                // No code generation needed here
            }
        }
    }

    fn gen_expr(&mut self, e: &Expression, dest_reg: u32) {
        match e {
            Expression::Number(n) => {
                self.emit_li(dest_reg, *n as u32);
            }
            
            Expression::Variable(name) => {
                // Check if it's a root symbol (constant)
                if let Some(&val) = self.root_symbols.get(name) {
                    // Root symbols are constants - use the value directly
                    self.emit_li(dest_reg, val);
                } else if let Some(&addr) = self.symbols.get(name) {
                    // Check if this is an array - if so, pass the address
                    if self.array_names.contains(name) {
                        // Arrays: load the address itself (for passing to functions)
                        self.emit_li(dest_reg, addr);
                    } else {
                        // Regular variables - load from memory
                        let addr_reg = self.alloc_reg();
                        self.emit_li(addr_reg, addr);
                        
                        // lw dest_reg, 0(addr_reg)
                        self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                        
                        self.free_reg(addr_reg);
                    }
                } else if let Some(&offset) = self.current_params.get(name) {
                    // Function parameters - load from stack
                    // lw dest_reg, offset($sp)
                    self.emit(0x8FA00000 | (dest_reg << 16) | (offset & 0xFFFF));
                }
            }

            Expression::ArrayAccess(name, index_expr) => {
                // Calculate index * 4 (word size)
                let idx_reg = self.alloc_reg();
                self.gen_expr(index_expr, idx_reg);
                
                // sll idx_reg, idx_reg, 2 (multiply by 4)
                self.emit(0x00000000 | (idx_reg << 16) | (idx_reg << 11) | (2 << 6));
                
                // Get base address - check if global array or function parameter
                let base_reg = self.alloc_reg();
                
                if let Some(&base_addr) = self.symbols.get(name) {
                    // Global array - load immediate address
                    self.emit_li(base_reg, base_addr);
                } else if let Some(&offset) = self.current_params.get(name) {
                    // Function parameter - load address from stack
                    // lw base_reg, offset($sp)
                    self.emit(0x8FA00000 | (base_reg << 16) | (offset & 0xFFFF));
                } else {
                    panic!("Array not found: {}", name);
                }
                
                // addu base_reg, base_reg, idx_reg
                self.emit(0x00000021 | (base_reg << 21) | (idx_reg << 16) | (base_reg << 11));
                
                // Load word from calculated address
                // lw dest_reg, 0(base_reg)
                self.emit(0x8C000000 | (base_reg << 21) | (dest_reg << 16));
                
                self.free_reg(base_reg);
                self.free_reg(idx_reg);
            }

            Expression::Peek(addr_expr) => {
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                
                // lb dest_reg, 0(addr_reg) - load byte
                self.emit(0x80000000 | (addr_reg << 21) | (dest_reg << 16));
                
                self.free_reg(addr_reg);
            }

            Expression::BinaryOp(left, op, right) => {
                let left_reg = self.alloc_reg();
                self.gen_expr(left, left_reg);
                
                let right_reg = self.alloc_reg();
                self.gen_expr(right, right_reg);
                
                match op.as_str() {
                    "+" => {
                        // addu dest_reg, left_reg, right_reg
                        self.emit(0x00000021 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                    }
                    "-" => {
                        // subu dest_reg, left_reg, right_reg
                        self.emit(0x00000023 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                    }
                    "*" => {
                        // mult left_reg, right_reg
                        self.emit(0x00000018 | (left_reg << 21) | (right_reg << 16));
                        // mflo dest_reg
                        self.emit(0x00000012 | (dest_reg << 11));
                    }
                    "/" => {
                        // div left_reg, right_reg
                        self.emit(0x0000001A | (left_reg << 21) | (right_reg << 16));
                        // mflo dest_reg
                        self.emit(0x00000012 | (dest_reg << 11));
                    }
                    "&" => {
                        // and dest_reg, left_reg, right_reg
                        self.emit(0x00000024 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                    }
                    "|" => {
                        // or dest_reg, left_reg, right_reg
                        self.emit(0x00000025 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                    }
                    "<" => {
                        // slt dest_reg, left_reg, right_reg (set if less than)
                        self.emit(0x0000002A | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                    }
                    ">" => {
                        // slt dest_reg, right_reg, left_reg (swap operands)
                        self.emit(0x0000002A | (right_reg << 21) | (left_reg << 16) | (dest_reg << 11));
                    }
                    "==" => {
                        // xor dest_reg, left_reg, right_reg
                        self.emit(0x00000026 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                        // sltiu dest_reg, dest_reg, 1 (set if equal to zero)
                        self.emit(0x2C000001 | (dest_reg << 21) | (dest_reg << 16));
                    }
                    _ => {}
                }
                
                self.free_reg(right_reg);
                self.free_reg(left_reg);
            }

            Expression::WaitKey | Expression::Inb(_) => {
                // I/O operations not yet implemented
                self.emit_li(dest_reg, 0);
            }
        }
    }

    // ===== HELPER FUNCTIONS =====
    
    fn emit(&mut self, instr: u32) {
        self.code.push(instr);
    }

    fn emit_li(&mut self, reg: u32, imm: u32) {
        let hi = (imm >> 16) & 0xFFFF;
        let lo = imm & 0xFFFF;
        
        if hi == 0 {
            // ori reg, $zero, lo
            self.emit(0x34000000 | (reg << 16) | lo);
        } else {
            // lui reg, hi
            self.emit(0x3C000000 | (reg << 16) | hi);
            if lo != 0 {
                // ori reg, reg, lo
                self.emit(0x34000000 | (reg << 21) | (reg << 16) | lo);
            }
        }
    }

    fn alloc_reg(&mut self) -> u32 {
        self.reg_pool.pop_front().unwrap_or(8)
    }

    fn free_reg(&mut self, reg: u32) {
        if (8..=15).contains(&reg) {
            self.reg_pool.push_back(reg);
        }
    }
}