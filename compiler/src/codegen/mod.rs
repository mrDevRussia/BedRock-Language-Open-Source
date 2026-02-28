use crate::ast::{Statement, Expression};
use std::collections::{HashMap, HashSet, VecDeque};

// ✅ CONSTANT: The physical start address of our code in RAM
const BASE_ADDR: u32 = 0x80000000;

pub struct Codegen { 
    code: Vec<u32>, 
    symbols: HashMap<String, u32>,      
    root_symbols: HashMap<String, u32>, 
    array_names: HashSet<String>,       
    functions: HashMap<String, usize>,  
    loop_stack: Vec<LoopContext>,       
    current_params: HashMap<String, u32>,
    local_vars: HashMap<String, u32>,   
    reg_pool: VecDeque<u32>,
    next_addr: u32,                     
    static_data: Vec<(u32, Vec<u8>)>,   
    in_function: bool,                  
    local_offset: u32,                  
}

struct LoopContext {
    start_addr: usize,
    break_patches: Vec<usize>,
}

impl Codegen {
    pub fn new() -> Self {
        let mut pool = VecDeque::new();
        for i in 8..=15 { pool.push_back(i); } 
        Codegen { 
            code: Vec::new(), 
            symbols: HashMap::new(),
            root_symbols: HashMap::new(),
            array_names: HashSet::new(),
            functions: HashMap::new(), 
            loop_stack: Vec::new(), 
            current_params: HashMap::new(),
            local_vars: HashMap::new(),
            reg_pool: pool, 
            next_addr: BASE_ADDR, // Start addresses after code? Will be adjusted dynamically usually
            static_data: Vec::new(),
            in_function: false,
            local_offset: 0,
        }
    }

    // ✅ HELPER: Convert Vector Index to MIPS Jump Target (Absolute Address >> 2)
    fn get_jump_target(&self, index: usize) -> u32 {
        let absolute_addr = BASE_ADDR + (index as u32 * 4);
        (absolute_addr >> 2) & 0x03FFFFFF
    }

    pub fn compile(&mut self, stmts: &[Statement]) -> Vec<u8> {
        // ============================================
        // Bootstrap Code (MIPS Initialization)
        // ============================================
        self.emit(0x00000000);  // nop
        // Initialize Stack Pointer ($sp) to 0x80101000
        self.emit(0x3C1D8010);  // lui $sp, 0x8010
        self.emit(0x37BD1000);  // ori $sp, $sp, 0x1000
        
        let global_jump_patch = self.code.len();
        self.emit(0x08000000);  // j (will be patched to init code)
        self.emit(0x00000000);  // nop (delay slot)

        // ============================================
        // Phase 1: Collect Root Symbols
        // ============================================
        for s in stmts {
            if let Statement::Root(name, expr) = s {
                if let Expression::Number(val) = expr {
                    self.root_symbols.insert(name.clone(), *val as u32);
                }
            }
        }

        // ============================================
        // Phase 2: Static Data Allocation
        // ============================================
        // Note: We place static data after a theoretical code size limit or calculate strictly
        // For now, let's keep your logic but ensure next_addr doesn't overlap code if possible.
        // A safer way is to set next_addr to a high value like 0x80004000 for data.
        self.next_addr = 0x80010000; // Move data area further away to avoid collision with code

        for s in stmts {
            match s {
                Statement::ArrayDefine(name, vals) => {
                    let addr = self.next_addr;
                    self.symbols.insert(name.clone(), addr);
                    self.array_names.insert(name.clone());
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
                    self.array_names.insert(name.clone());
                    let mut bytes = s_val.as_bytes().to_vec();
                    bytes.push(0);
                    let bytes_len = bytes.len();
                    self.static_data.push((addr, bytes));
                    self.next_addr += bytes_len as u32;
                    self.next_addr = (self.next_addr + 3) & !3; // Align
                }
                _ => {}
            }
        }

        // ============================================
        // Phase 3: Function Code Generation
        // ============================================
        for s in stmts {
            if let Statement::FunctionDefine(_, _, _) = s {
                self.generate_stmt(s);
            }
        }

        // ============================================
        // Phase 4: Patch Global Jump to Init Code
        // ============================================
        let init_start_idx = self.code.len();
        // ✅ FIXED: Use get_jump_target
        self.code[global_jump_patch] = 0x08000000 | self.get_jump_target(init_start_idx);

        // ============================================
        // Phase 5: Static Data Initialization
        // ============================================
        let static_data_copy = self.static_data.clone();
        for (addr, bytes) in static_data_copy.iter() {
            let mut offset = 0;
            for chunk in bytes.chunks(4) {
                let mut value = 0u32;
                for (i, &byte) in chunk.iter().enumerate() {
                    value |= (byte as u32) << ((3 - i) * 8);
                }
                let val_reg = self.alloc_reg();
                self.emit_li(val_reg, value);
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, addr + offset);
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); // sw
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
                offset += 4;
            }
        }

        // ============================================
        // Phase 6: Global Code Generation
        // ============================================
        for s in stmts {
            if !matches!(s, Statement::Root(, _) | Statement::FunctionDefine(, _, _) 
                           | Statement::ArrayDefine(, _) | Statement::StringDefine(, _)) {
                self.generate_stmt(s);
            }
        }

        // ============================================
        // Phase 7: Halt (Infinite Loop)
        // ============================================
        let halt_idx = self.code.len();
        // ✅ FIXED: Jump to self (infinite loop) using correct address
        self.emit(0x08000000 | self.get_jump_target(halt_idx));
        self.emit(0x00000000);

        // ============================================
        // Phase 8: Binary Generation (Big-Endian)
        // ============================================
        let mut binary = Vec::new();
        for &instr in &self.code {
            binary.extend_from_slice(&instr.to_be_bytes());
        }
        binary
    }

    fn generate_stmt(&mut self, s: &Statement) {
        match s {
            Statement::Poke(addr_expr, val_expr) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); // sw
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }
            
            Statement::Outb(addr_expr, val_expr) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                // sb instruction (Store Byte) - Opcode 0xA0
                self.emit(0xA0000000 | (addr_reg << 21) | (val_reg << 16)); 
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }

            Statement::ArrayDefine(, _) | Statement::StringDefine(, _) => {}

            Statement::FunctionDefine(name, params, body) => {
                self.functions.insert(name.clone(), self.code.len());
                
                self.in_function = true;
                self.local_offset = 0;
                self.local_vars.clear();
                
                // Prologue: Save $ra
                self.emit(0x27BDFFE0); // addiu $sp, $sp, -32
                self.emit(0xAFBF001C); // sw $ra, 28($sp)
                
                self.current_params.clear();
                // Map params to stack locations ABOVE the new frame
                // Caller puts args, then does jal. $sp is decremented by callee.
                // So args are at $sp + 32 + (arg_index * 4)
                for (i, p) in params.iter().enumerate() {
                    self.current_params.insert(p.clone(), 32 + (i * 4) as u32);
                }
                
                for stmt in body { self.generate_stmt(stmt); }
                
                // Epilogue: Restore $ra and return
                self.emit(0x8FBF001C); // lw $ra, 28($sp)
                self.emit(0x27BD0020); // addiu $sp, $sp, 32
                self.emit(0x03E00008); // jr $ra
                self.emit(0x00000000); // nop
                
                self.in_function = false;
                self.current_params.clear();
            }

            Statement::Call(func_name, args) => {
                // 1. Evaluate args
                let mut arg_regs = Vec::new();
                for arg in args.iter() {
                    let arg_reg = self.alloc_reg();
                    self.gen_expr(arg, arg_reg);
                    arg_regs.push(arg_reg);
                }
                
                // 2. Reserve stack space for args
                if !args.is_empty() {
                    let arg_space = (args.len() * 4) as u32;
                    let neg_space = (-(arg_space as i32)) as u32; // Two's complement
                    self.emit(0x27BD0000 | (neg_space & 0xFFFF)); // addiu $sp, $sp, -size
                }
                
                // 3. Store args on stack
                for (i, &arg_reg) in arg_regs.iter().enumerate() {
                    self.emit(0xAFA00000 | (arg_reg << 16) | ((i * 4) as u32 & 0xFFFF)); // sw reg, offset($sp)
                    self.free_reg(arg_reg);
                }
                
                // 4. JAL to Function
                if let Some(&func_idx) = self.functions.get(func_name) {
                    // ✅ FIXED: Calculate correct MIPS jump target
                    let target = self.get_jump_target(func_idx);
                    self.emit(0x0C000000 | target); // jal target
                    self.emit(0x00000000); // nop
                }
                
                // 5. Restore stack space
                if !args.is_empty() {
                    let arg_space = (args.len() * 4) as u32;
                    self.emit(0x27BD0000 | (arg_space & 0xFFFF)); // addiu $sp, $sp, size
                }
            }

            Statement::Let(name, value) | Statement::Assignment(name, value) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(value, val_reg);
                
                if self.in_function {
                    let offset = if let Some(&existing_offset) = self.local_vars.get(name) {
                        existing_offset
                    } else {
                        self.local_offset += 4;
                        let offset = self.local_offset;
                        self.local_vars.insert(name.clone(), offset);
                        offset
                    };
                    // sw val_reg, offset($sp) -- using $sp (29) which is 0x1D
                    // Opcode sw is 0xAC (not 0xAFA0.. AFA0 is sw $zero?)
                    // Fixed: sw rt, offset(base) -> 0xAC | base<<21 | rt<<16 | offset
                    // base is $sp (29) -> 11101
                    self.emit(0xAFBD0000 | (val_reg << 16) | (offset & 0xFFFF));
                } else {
                    let addr = *self.symbols.entry(name.clone()).or_insert_with(|| {
                        let a = self.next_addr;
                        self.next_addr += 4;
                        a
                    });
                    let addr_reg = self.alloc_reg();
                    self.emit_li(addr_reg, addr);
                    self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16));
                    self.free_reg(addr_reg);
                }
                self.free_reg(val_reg);
            }

            Statement::Loop(body) => {
                let start_idx = self.code.len();
                self.loop_stack.push(LoopContext { start_addr: start_idx, break_patches: Vec::new() });
                
                for stmt in body { self.generate_stmt(stmt); }
                
                // Jump back to start
                // ✅ FIXED: Use absolute jump with correct address
                self.emit(0x08000000 | self.get_jump_target(start_idx));
                self.emit(0x00000000);
                
                if let Some(ctx) = self.loop_stack.pop() {
                    let end_idx = self.code.len();
                    for patch_pos in ctx.break_patches {
                        // Branch offset (relative instructions) is still correct as is
                        let offset = (end_idx as i32 - patch_pos as i32 - 1) as i16;
                        self.code[patch_pos] = 0x10000000 | ((offset as u16) as u32);
                    }
                }
            }

            Statement::While(cond, body) => {
                let start_idx = self.code.len();
                self.loop_stack.push(LoopContext { start_addr: start_idx, break_patches: Vec::new() });
                
                let cond_reg = self.alloc_reg();
                self.gen_expr(cond, cond_reg);
                
                let branch_patch = self.code.len();
                // beq $cond, $zero, end (jump if false)
                self.emit(0x10000000 | (cond_reg << 21)); 
                self.emit(0x00000000);
                
                for stmt in body { self.generate_stmt(stmt); }
                
                // Jump back to start
                // ✅ FIXED: Use absolute jump
                self.emit(0x08000000 | self.get_jump_target(start_idx));
                self.emit(0x00000000);
                
                let end_idx = self.code.len();
                let offset = (end_idx as i32 - branch_patch as i32 - 1) as i16;
                self.code[branch_patch] = 0x10000000 | (cond_reg << 21) | ((offset as u16) as u32);
                
                self.free_reg(cond_reg);
                
                if let Some(ctx) = self.loop_stack.pop() {
                    for patch_pos in ctx.break_patches {
                        let offset = (end_idx as i32 - patch_pos as i32 - 1) as i16;
                        self.code[patch_pos] = 0x10000000 | ((offset as u16) as u32);
                    }
                }
            }

            Statement::If(cond, then_body, else_body) => {
                let cond_reg = self.alloc_reg();
                self.gen_expr(cond, cond_reg);
                
                let branch_patch = self.code.len();
                self.emit(0x10000000 | (cond_reg << 21)); // beq
                self.emit(0x00000000);
                self.free_reg(cond_reg);
                
                for stmt in then_body { self.generate_stmt(stmt); }
                
                if let Some(else_stmts) = else_body {
                    let jump_over_else_patch = self.code.len();
                    self.emit(0x08000000); // Placeholder jump
                    self.emit(0x00000000);
                    
                    let else_start_idx = self.code.len();
                    let offset = (else_start_idx as i32 - branch_patch as i32 - 1) as i16;
                    self.code[branch_patch] = 0x10000000 | (cond_reg << 21) | ((offset as u16) as u32);
                    
                    for stmt in else_stmts { self.generate_stmt(stmt); }
                    
                    let end_idx = self.code.len();
                    // ✅ FIXED: Patch the jump over else
                    self.code[jump_over_else_patch] = 0x08000000 | self.get_jump_target(end_idx);
                } else {
                    let end_idx = self.code.len();
                    let offset = (end_idx as i32 - branch_patch as i32 - 1) as i16;
                    self.code[branch_patch] = 0x10000000 | (cond_reg << 21) | ((offset as u16) as u32);
                }
            }

            Statement::Break => {
                if let Some(ctx) = self.loop_stack.last_mut() {
                    let patch_pos = self.code.len();
                    ctx.break_patches.push(patch_pos);
                    self.emit(0x10000000); // beq $zero, $zero, offset (unconditional branch)
                    self.emit(0x00000000);
                }
            }

            Statement::Return => {
                self.emit(0x8FBF001C); // lw $ra, 28($sp)
                self.emit(0x27BD0020); // addiu $sp, $sp, 32
                self.emit(0x03E00008); // jr $ra
                self.emit(0x00000000);
            }

            Statement::Root(_, _) => {}
            Statement::Asm(code_str) => {
                 // Simple hex parser for manual ASM injection if needed
                 // e.g., asm("00000000") -> nop
                 if let Ok(instr) = u32::from_str_radix(&code_str, 16) {
                     self.emit(instr);
                 }
            }
        }
    }

    fn gen_expr(&mut self, expr: &Expression, dest_reg: u32) {
        match expr {
            Expression::Number(n) => {
                self.emit_li(dest_reg, *n as u32);
            }

            Expression::Variable(name) => {
                if let Some(&addr) = self.root_symbols.get(name) {
                    self.emit_li(dest_reg, addr);
                } else if self.in_function {
                    if let Some(&offset) = self.current_params.get(name) {
                        self.emit(0x8FBD0000 | (dest_reg << 16) | (offset & 0xFFFF)); // lw from $sp
                    } else if let Some(&offset) = self.local_vars.get(name) {
                        self.emit(0x8FBD0000 | (dest_reg << 16) | (offset & 0xFFFF)); // lw from $sp
                    } else {
                        // Global variable
                        let addr = *self.symbols.get(name).unwrap_or(&0x80010000);
                        let addr_reg = self.alloc_reg();
                        self.emit_li(addr_reg, addr);
                        self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                        self.free_reg(addr_reg);
                    }
                } else {
                    let addr = *self.symbols.get(name).unwrap_or(&0x80010000);
                    let addr_reg = self.alloc_reg();
                    self.emit_li(addr_reg, addr);
                    self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                    self.free_reg(addr_reg);
                }
            }

            Expression::ArrayAccess(name, index_expr) => {
                let idx_reg = self.alloc_reg();
                self.gen_expr(index_expr, idx_reg);
                
                // sll idx_reg, idx_reg, 2 (multiply by 4)
                self.emit(0x00000000 | (idx_reg << 16) | (idx_reg << 11) | (2 << 6)); 
                
                let base_addr = *self.symbols.get(name).unwrap_or(&0x80010000);
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, base_addr);
                
                // add addr_reg, addr_reg, idx_reg
                self.emit(0x00000020 | (addr_reg << 21) | (idx_reg << 16) | (addr_reg << 11) | 0x20); // add
                
                // lw dest_reg, 0(addr_reg)
                self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                
                self.free_reg(addr_reg);
                self.free_reg(idx_reg);
            }

            Expression::Peek(addr_expr) => {
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                self.free_reg(addr_reg);
            }

            Expression::BinaryOp(left, op, right) => {
                let left_reg = self.alloc_reg();
                self.gen_expr(left, left_reg);
                let right_reg = self.alloc_reg();
                self.gen_expr(right, right_reg);
                
                match op.as_str() {
                    "+" => self.emit(0x00000020 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11) | 0x20), // add
                    "-" => self.emit(0x00000022 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11) | 0x22), // sub
                    "*" => { // mult
                        self.emit(0x00000018 | (left_reg << 21) | (right_reg << 16));
                        self.emit(0x00004812 | (dest_reg << 11)); // mflo
                    }
                    "/" => { // div
                        self.emit(0x0000001A | (left_reg << 21) | (right_reg << 16));
                        self.emit(0x00004812 | (dest_reg << 11)); // mflo
                    }
                    "&" => self.emit(0x00000024 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11) | 0x24), // and
                    "|" => self.emit(0x00000025 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11) | 0x25), // or
                    // For logic ops, we store 1 or 0
                    "==" => {
                        // xor temp, left, right (0 if equal)
                        self.emit(0x00000026 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11) | 0x26);
                        // sltiu dest, dest, 1 (if 0 -> 1, else -> 0)
                        self.emit(0x2C000001 | (dest_reg << 21) | (dest_reg << 16));
                    }
                     ">" => {
                        // slt dest, right, left
                        self.emit(0x0000002A | (right_reg << 21) | (left_reg << 16) | (dest_reg << 11) | 0x2A);
                    }
                    "<" => {
                         // slt dest, left, right
                        self.emit(0x0000002A | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11) | 0x2A);
                    }
                    _ => {}
                }
                self.free_reg(right_reg);
                self.free_reg(left_reg);
            }

            Expression::WaitKey | Expression::Inb(_) => {
                self.emit_li(dest_reg, 0);
            }
        }
    }

    fn emit(&mut self, instr: u32) {
        self.code.push(instr);
    }

    fn emit_li(&mut self, reg: u32, imm: u32) {
        let hi = (imm >> 16) & 0xFFFF;
        let lo = imm & 0xFFFF;
        if hi == 0 {
            self.emit(0x34000000 | (reg << 16) | lo); // ori
        } else {
            self.emit(0x3C000000 | (reg << 16) | hi); // lui
            if lo != 0 {
                self.emit(0x34000000 | (reg << 21) | (reg << 16) | lo); // ori
            }
        }
    }

    fn alloc_reg(&mut self) -> u32 {
        self.reg_pool.pop_front().unwrap_or(8)
    }

    fn free_reg(&mut self, reg: u32) {
        if (8..=15).contains(&reg) && !self.reg_pool.contains(&reg) {
            self.reg_pool.push_back(reg);
        }
    }
}