use crate::ast::{Statement, Expression};
use std::collections::{HashMap, VecDeque};
use serde::Serialize;

const BASE_ADDR: u32 = 0x80000000;

#[derive(Debug, Clone, Serialize)]
pub struct SourceMapEntry {
    pub line: usize,
    pub address: u32,
    pub instruction: u32,
    pub source: String,
}

pub struct Codegen { 
    base_addr: u32,  
    code: Vec<u32>, 
    symbols: HashMap<String, u32>,      
    root_symbols: HashMap<String, u32>, 
    functions: HashMap<String, usize>,  
    loop_stack: Vec<LoopContext>,   
    current_func_exit: Option<usize>,   
    current_params: HashMap<String, u32>,
    local_vars: HashMap<String, u32>,   
    reg_pool: VecDeque<u32>,
    next_addr: u32,                     
    static_data: Vec<(u32, Vec<u8>)>,   
    in_function: bool,                  
    local_offset: u32,
    source_map: Vec<SourceMapEntry>,
    current_line: usize,
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
            base_addr: BASE_ADDR,
            code: Vec::new(), 
            symbols: HashMap::new(),
            root_symbols: HashMap::new(),
            functions: HashMap::new(), 
            loop_stack: Vec::new(), 
            current_func_exit: None,
            current_params: HashMap::new(),
            local_vars: HashMap::new(),
            reg_pool: pool,
            next_addr: BASE_ADDR,
            static_data: Vec::new(),
            in_function: false,                  
            local_offset: 0,
            source_map: Vec::new(),
            current_line: 0,
        }
    }

    fn get_jump_target(&self, index: usize) -> u32 {
        let absolute_addr = self.base_addr + (index as u32 * 4); 
        (absolute_addr >> 2) & 0x03FFFFFF
    }

    pub fn compile(&mut self, stmts: &[Statement]) -> Vec<u8> {
    self.code.clear();
        self.symbols.clear();
        self.static_data.clear();
        self.root_symbols.clear();
        self.source_map.clear();
        for s in stmts {
            if let Statement::Root(name, Expression::Number(val)) = s {
                self.root_symbols.insert(name.clone(), *val as u32);
            }
        }
                
        self.base_addr = *self.root_symbols.get("BASE").unwrap_or(&0x80000000);
        self.next_addr = *self.root_symbols.get("DATA").unwrap_or(&(self.base_addr + 0x10000));
        let stack_ptr = *self.root_symbols.get("STACK").unwrap_or(&(self.base_addr + 0x20000));
            
        self.emit(0x00000000); 
        self.emit_li(29, stack_ptr); 

        let global_jump_patch = self.code.len(); 
        self.emit(0x08000000);
        self.emit(0x00000000);

        for s in stmts {
            match s {
                Statement::ArrayDefine(name, vals) => {
                    let addr = self.next_addr;
                    self.symbols.insert(name.clone(), addr);
                    let mut bytes = Vec::new();
                    for v in vals { bytes.extend_from_slice(&(*v as u32).to_be_bytes()); }
                    self.static_data.push((addr, bytes));
                    self.next_addr += (vals.len() * 4) as u32;
                }
                Statement::StringDefine(name, s_val) => {
                    let addr = self.next_addr;
                    self.symbols.insert(name.clone(), addr);
                    let mut bytes = s_val.as_bytes().to_vec();
                    bytes.push(0); // Null terminator
                    self.static_data.push((addr, bytes.clone()));
                    self.next_addr += ((bytes.len() + 3) & !3) as u32;
                }
                _ => {}
            }
        }

 
        for s in stmts {
            if let Statement::FunctionDefine(name, _, _) = s {
       
                self.functions.insert(name.clone(), 0);
            }
        }

   
        for s in stmts {
            if let Statement::FunctionDefine(_, _, _) = s {
                self.generate_stmt(s);
            }
        }

        let init_start_idx = self.code.len(); 
        let target = self.get_jump_target(init_start_idx); 
        self.code[global_jump_patch] = 0x08000000 | target;

        let static_data_copy = self.static_data.clone();
        for (addr, bytes) in static_data_copy {
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
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); // sw $val, 0($addr)
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
                offset += 4;
            }
        }

        for s in stmts {
            if !matches!(s, Statement::Root(_, _) | Statement::FunctionDefine(_, _, _) 
               | Statement::ArrayDefine(_, _) | Statement::StringDefine(_, _)) {
                self.generate_stmt(s);
            }
        }

        let halt_idx = self.code.len();
        self.emit(0x08000000 | self.get_jump_target(halt_idx)); 
        self.emit(0x00000000);

        self.code.iter().flat_map(|&instr| instr.to_be_bytes().to_vec()).collect()
    }

 pub fn set_current_line(&mut self, line: usize) {
        self.current_line = line;
    }

   fn generate_stmt(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let(_, _) | Statement::Assignment(_, _) |
            Statement::Call(_, _) | Statement::Return(_) |
            Statement::Poke(_, _) | Statement::Outb(_, _) |
            Statement::Asm(_) | Statement::Break | Statement::CallPtr(_) |
            Statement::ArrayAssign(_, _, _) => {
                self.current_line += 1;
            }
            _ => {}
        }

        match stmt {
            Statement::FunctionDefine(name, params, body) => {
                self.functions.insert(name.clone(), self.code.len());
                self.in_function = true;
                self.local_offset = 0;
                self.local_vars.clear();

                self.emit(0x27BDFFE0);
                self.emit(0xAFBF001C);

                self.current_params.clear();
                for (i, p) in params.iter().enumerate() {
                    self.current_params.insert(p.clone(), 32 + (i * 4) as u32);
                }

                let exit_patch_idx = self.code.len();
                self.emit(0x08000000);
                self.emit(0x00000000);
                self.current_func_exit = Some(exit_patch_idx);

                for s in body { self.generate_stmt(s); }

                let epilogue_idx = self.code.len();
           
                self.code[exit_patch_idx] = 0x08000000 | self.get_jump_target(epilogue_idx);

                self.emit(0x8FBF001C);
                self.emit(0x27BD0020);
                self.emit(0x03E00008);
                self.emit(0x00000000);

                self.in_function = false;
                self.current_func_exit = None;
            }

            Statement::Call(func_name, args) => {
                let mut arg_regs = Vec::new();
                for arg in args {
                    let r = self.alloc_reg();
                    self.gen_expr(arg, r);
                    arg_regs.push(r);
                }
                if !args.is_empty() {
                    let space = (args.len() * 4) as u32;
                    let neg_space = (-(space as i32)) as u32;
                    self.emit(0x27BD0000 | (29 << 21) | (29 << 16) | (neg_space & 0xFFFF));
                }
                for (i, &reg) in arg_regs.iter().enumerate() {
                    self.emit(0xAC000000 | (29 << 21) | (reg << 16) | ((i * 4) as u32 & 0xFFFF));
                    self.free_reg(reg);
                }
                if let Some(&idx) = self.functions.get(func_name) {
                    let target = self.get_jump_target(idx);
                    self.emit(0x0C000000 | target); // jal
                    self.emit(0x00000000);
                }
                if !args.is_empty() {
                    let space = (args.len() * 4) as u32;
                    self.emit(0x27BD0000 | (29 << 21) | (29 << 16) | (space & 0xFFFF));
                }
            }

            Statement::Let(name, value) | Statement::Assignment(name, value) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(value, val_reg);
                if self.in_function {
                    let offset = *self.local_vars.entry(name.clone()).or_insert_with(|| {
                        self.local_offset += 4; self.local_offset
                    });
                    self.emit(0xAFA00000 | (val_reg << 16) | (offset & 0xFFFF)); // sw $val, offset($sp)
                } else {
                    let addr = *self.symbols.entry(name.clone()).or_insert_with(|| {
                        let a = self.next_addr; self.next_addr += 4; a
                    });
                    let addr_reg = self.alloc_reg();
                    self.emit_li(addr_reg, addr);
                    self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); // sw $val, 0($addr)
                    self.free_reg(addr_reg);
                }
                self.free_reg(val_reg);
            }

            Statement::ArrayAssign(name, index_expr, val_expr) => {
                let idx_reg = self.alloc_reg();
                self.gen_expr(index_expr, idx_reg);
                self.emit(0x00000000 | (idx_reg << 16) | (idx_reg << 11) | (2 << 6)); // sll idx, idx, 2
                
                let base_addr = *self.symbols.get(name).unwrap_or(&0x80010000);
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, base_addr);
                self.emit(0x00000021 | (addr_reg << 21) | (idx_reg << 16) | (addr_reg << 11)); // addu
                
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); // sw $val, 0($addr)
                
                self.free_reg(val_reg);
                self.free_reg(addr_reg);
                self.free_reg(idx_reg);
            }

            Statement::If(cond, then_body, else_body) => {
                let cond_reg = self.alloc_reg();
                self.gen_expr(cond, cond_reg);
                let b_patch = self.code.len();
                self.emit(0x10000000 | (cond_reg << 21)); // beq $cond, $zero, offset
                self.emit(0x00000000);
                self.free_reg(cond_reg);

                for s in then_body { self.generate_stmt(s); }

                if let Some(else_stmts) = else_body {
                    let j_patch = self.code.len();
                    self.emit(0x08000000); // Jump over else
                    self.emit(0x00000000);
                    let else_start = self.code.len();
                    self.code[b_patch] |= ((else_start as i32 - b_patch as i32 - 2) as u16) as u32;
                    for s in else_stmts { self.generate_stmt(s); }
                    let end = self.code.len();
                    self.code[j_patch] |= self.get_jump_target(end);
                } else {
                    let end = self.code.len();
                    self.code[b_patch] |= ((end as i32 - b_patch as i32 - 1) as u16) as u32;
                }
            }

            Statement::While(cond, body) => {
                let start = self.code.len();
                let cond_reg = self.alloc_reg();
                self.gen_expr(cond, cond_reg);
                
                let exit_patch = self.code.len();
                self.emit(0x10000000 | (cond_reg << 21)); // beq $cond, $zero, exit
                self.emit(0x00000000);
                self.free_reg(cond_reg);

                self.loop_stack.push(LoopContext { start_addr: start, break_patches: Vec::new() });
                for s in body { self.generate_stmt(s); }
                
                self.emit(0x08000000 | self.get_jump_target(start)); // jump to start
                self.emit(0x00000000);
                
                let end = self.code.len();
                self.code[exit_patch] |= ((end as i32 - exit_patch as i32 - 1) as u16) as u32;
                if let Some(ctx) = self.loop_stack.pop() {
                    for pos in ctx.break_patches {
                        self.code[pos] |= ((end as i32 - pos as i32 - 1) as u16) as u32;
                    }
                }
            }

            Statement::Loop(body) => {
                let start = self.code.len();
                self.loop_stack.push(LoopContext { start_addr: start, break_patches: Vec::new() });
                for s in body { self.generate_stmt(s); }
                self.emit(0x08000000 | self.get_jump_target(start));
                self.emit(0x00000000);
                if let Some(ctx) = self.loop_stack.pop() {
                    let end = self.code.len();
                    for pos in ctx.break_patches {
                        self.code[pos] |= ((end as i32 - pos as i32 - 1) as u16) as u32;
                    }
                }
            }

            Statement::Break => {
                if let Some(ctx) = self.loop_stack.last_mut() {
                    let pos = self.code.len();
                    ctx.break_patches.push(pos);
                    self.emit(0x10000000); // beq $0, $0 (unconditional branch)
                    self.emit(0x00000000);
                }
            }

            Statement::Return(maybe_expr) => {
                if let Some(expr) = maybe_expr { self.gen_expr(expr, 2); } // $v0 = 2
                if let Some(exit) = self.current_func_exit {
                    self.emit(0x08000000 | self.get_jump_target(exit));
                    self.emit(0x00000000);
                }
            }

            Statement::Poke(addr_expr, val_expr) => {
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); // sw $val, 0($addr)
                self.free_reg(val_reg);
                self.free_reg(addr_reg);
            }

            Statement::Outb(port_expr, val_expr) => {
        
                let port_reg = self.alloc_reg();
                self.gen_expr(port_expr, port_reg);
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                self.emit(0xAC000000 | (port_reg << 21) | (val_reg << 16)); 
                self.free_reg(val_reg);
                self.free_reg(port_reg);
            }

            Statement::Asm(hex) => {
                if let Ok(instr) = u32::from_str_radix(hex, 16) { self.emit(instr); }
            }
            Statement::CallPtr(expr) => {
                let reg = self.alloc_reg();
                self.gen_expr(expr, reg);
                let jalr = 0x00000009u32
                    | (reg << 21)
                    | (31 << 11);
                self.emit(jalr);
                self.emit(0x00000000);
                self.free_reg(reg);
            }
            _ => {}
        }
    }

    fn gen_expr(&mut self, expr: &Expression, dest_reg: u32) {
        match expr {
            Expression::Number(n) => { self.emit_li(dest_reg, *n as u32); }

            Expression::Variable(name) => {
                if let Some(&addr) = self.root_symbols.get(name) {
                    self.emit_li(dest_reg, addr);
                } else if self.in_function {
                    if let Some(&offset) = self.current_params.get(name) {
                        self.emit(0x8C000000 | (29 << 21) | (dest_reg << 16) | (offset & 0xFFFF)); // lw from param
                    } else if let Some(&offset) = self.local_vars.get(name) {
                        self.emit(0x8C000000 | (29 << 21) | (dest_reg << 16) | (offset & 0xFFFF)); // lw from local
                    } else {
                        let addr = *self.symbols.get(name).unwrap_or(&0x80010000);
                        let addr_reg = self.alloc_reg();
                        self.emit_li(addr_reg, addr);
                        self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                        self.free_reg(addr_reg);
                    }
                } else {
                    let addr = *self.symbols.get(name).expect("Variable not defined!");
                    let addr_reg = self.alloc_reg();
                    self.emit_li(addr_reg, addr);
                    self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                    self.free_reg(addr_reg);
                }
            }

            Expression::BinaryOp(left, op, right) => {
                let left_reg = self.alloc_reg();
                self.gen_expr(left, left_reg);
                let right_reg = self.alloc_reg();
                self.gen_expr(right, right_reg);
                
                match op.as_str() {
                    "+" => self.emit(0x00000021 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)), // addu
                    "-" => self.emit(0x00000023 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)), // subu
                    "*" => {
                        self.emit(0x00000018 | (left_reg << 21) | (right_reg << 16)); // mult
                        self.emit(0x00000012 | (dest_reg << 11)); // mflo
                    }


"<<" => {
   
    self.emit(0x00000004 | (right_reg << 21) | (left_reg << 16) | (dest_reg << 11));
}
">>" => {
   
    self.emit(0x00000006 | (right_reg << 21) | (left_reg << 16) | (dest_reg << 11));
}


                    "^" => {
                        self.emit(0x00000026 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11));
                     }
                    "/" => {
                        self.emit(0x0000001A | (left_reg << 21) | (right_reg << 16)); // div
                        self.emit(0x00000012 | (dest_reg << 11)); // mflo
                    }
                    "&" => self.emit(0x00000024 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)), // and
                    "|" => self.emit(0x00000025 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)), // or
                    "==" => {
                        self.emit(0x00000026 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)); // xor
                        self.emit(0x28000001 | (dest_reg << 21) | (dest_reg << 16)); // sltiu
                    }
                    "!=" => {
                        self.emit(0x00000026 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)); // xor
                        self.emit(0x0000002B | (0 << 21) | (dest_reg << 16) | (dest_reg << 11)); // sltu
                    }
                    ">"  => self.emit(0x0000002A | (right_reg << 21) | (left_reg << 16) | (dest_reg << 11)), // slt (swapped)
                    "<"  => self.emit(0x0000002A | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)), // slt
                    ">=" => {
                        self.emit(0x0000002A | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)); // slt
                        self.emit(0x38000001 | (dest_reg << 21) | (dest_reg << 16)); // xori $d, $d, 1
                    }
                    "<=" => {
                        self.emit(0x0000002A | (right_reg << 21) | (left_reg << 16) | (dest_reg << 11)); // slt
                        self.emit(0x38000001 | (dest_reg << 21) | (dest_reg << 16)); // xori $d, $d, 1
                    }
                    _ => {}
                }
                self.free_reg(right_reg);
                self.free_reg(left_reg);
            }

            Expression::ArrayAccess(name, index_expr) => {
                let idx_reg = self.alloc_reg();
                self.gen_expr(index_expr, idx_reg);
                self.emit(0x00000000 | (idx_reg << 16) | (idx_reg << 11) | (2 << 6)); // sll idx, idx, 2
                let base_addr = *self.symbols.get(name).unwrap_or(&0x80010000);
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, base_addr);
                self.emit(0x00000021 | (addr_reg << 21) | (idx_reg << 16) | (dest_reg << 11)); // addu
                self.emit(0x8C000000 | (dest_reg << 21) | (dest_reg << 16)); // lw $dest, 0($dest)
                self.free_reg(addr_reg);
                self.free_reg(idx_reg);
            }

            Expression::Peek(addr_expr) => {
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16)); // lw
                self.free_reg(addr_reg);
            }

            Expression::WaitKey => {
             
                self.emit_li(dest_reg, 0x80020000);
                self.emit(0x8C000000 | (dest_reg << 21) | (dest_reg << 16));
            }

            Expression::Inb(addr_expr) => {
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);
                self.emit(0x8C000000 | (addr_reg << 21) | (dest_reg << 16));
                self.free_reg(addr_reg);
            }

Expression::Call(name, args) => {
   
    let mut arg_regs = Vec::new();
    for arg in args {
        let r = self.alloc_reg();
        self.gen_expr(arg, r);
        arg_regs.push(r);
    }

    if !args.is_empty() {
        let space = (args.len() * 4) as u32;
        let neg_space = (-(space as i32)) as u32;
        self.emit(0x27BD0000 | (29 << 21) | (29 << 16) | (neg_space & 0xFFFF)); // addiu $sp, $sp, -space
        
        for (i, &reg) in arg_regs.iter().enumerate() {
            self.emit(0xAC000000 | (29 << 21) | (reg << 16) | ((i * 4) as u32 & 0xFFFF)); // sw $reg, offset($sp)
            self.free_reg(reg);
        }
    }

    if let Some(&idx) = self.functions.get(name) {
        let target = self.get_jump_target(idx);
        self.emit(0x0C000000 | target); // jal target
        self.emit(0x00000000);          // nop (delay slot)
    } else {
        panic!("Function '{}' not defined before use!", name);
    }

    self.emit(0x00400021 | (0 << 21) | (2 << 16) | (dest_reg << 11)); // addu $dest, $0, $v0

    if !args.is_empty() {
        let space = (args.len() * 4) as u32;
        self.emit(0x27BD0000 | (29 << 21) | (29 << 16) | (space & 0xFFFF)); // addiu $sp, $sp, space
    }
}

        }
    }

    fn emit(&mut self, instr: u32) {
        let address = self.base_addr + (self.code.len() as u32 * 4);
        self.source_map.push(SourceMapEntry {
            line: self.current_line,
            address,
            instruction: instr,
            source: String::new(),
        });
        self.code.push(instr);
    }

    fn emit_li(&mut self, reg: u32, imm: u32) {
        let hi = (imm >> 16) & 0xFFFF;
        let lo = imm & 0xFFFF;
        if hi == 0 {
            self.emit(0x34000000 | (0 << 21) | (reg << 16) | lo); // ori $reg, $0, lo
        } else {
            self.emit(0x3C000000 | (reg << 16) | hi); // lui $reg, hi
            if lo != 0 { self.emit(0x34000000 | (reg << 21) | (reg << 16) | lo); } // ori
        }
    }

pub fn get_source_map(&self) -> &Vec<SourceMapEntry> {
        &self.source_map
    }

    fn alloc_reg(&mut self) -> u32 { self.reg_pool.pop_front().unwrap_or(8) }
    fn free_reg(&mut self, reg: u32) {
        if (8..=15).contains(&reg) && !self.reg_pool.contains(&reg) {
            self.reg_pool.push_back(reg);
        }
    }
}