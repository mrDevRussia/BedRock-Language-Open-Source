use crate::ast::{Statement, Expression};
use std::collections::{HashMap, VecDeque};

pub struct Codegen { 
    code: Vec<u32>, 
    symbols: HashMap<String, u32>,
    functions: HashMap<String, usize>,
    loop_stack: Vec<Vec<usize>>,
    current_params: HashMap<String, u32>,
    reg_pool: VecDeque<u32>,
    next_addr: u32,
}

impl Codegen {
    pub fn new() -> Self {
        let mut pool = VecDeque::new();
        for i in 8..=15 { pool.push_back(i); } 
        Codegen { 
            code: Vec::new(), 
            symbols: HashMap::new(), 
            functions: HashMap::new(), 
            loop_stack: Vec::new(), 
            current_params: HashMap::new(), 
            reg_pool: pool, 
            next_addr: 0x80001000 
        }
    }

   pub fn compile(&mut self, stmts: &[Statement]) -> Vec<u8> {
        self.emit(0x40806000); 
        self.emit(0x3C1D8010); 
        self.emit(0x37BD0000); 

        let j_patch = self.code.len();
        self.emit(0x08000000); // مكان الـ Jump الافتتاحي

        // 1. تجميع الـ Roots أولاً
        for s in stmts {
            if let Statement::Root(name, expr) = s {
                if let Expression::Number(val) = expr {
                    self.symbols.insert(name.clone(), *val as u32);
                }
            }
        }

        // 2. تجميع كل الدوال بما فيها main
        for s in stmts {
            if let Statement::FunctionDefine(_, _, _) = s {
                self.generate_stmt(s);
            }
        }

        // 3. تصحيح الـ Jump ليذهب لعنوان دالة main الحقيقي
        if let Some(&main_pos) = self.functions.get("main") {
            let jump_target = main_pos as u32; // العنوان محسوب بالـ instructions
            self.code[j_patch] = 0x08000000 | (jump_target & 0x03FFFFFF);
        } else {
            // إذا لم يجد دالة main، يكمل تنفيذ ما بعد الهيدر مباشرة
            let jump_target = (self.code.len()) as u32;
            self.code[j_patch] = 0x08000000 | (jump_target & 0x03FFFFFF);
        }

        // 4. تجميع أي كود خارج الدوال (Global scope)
        for s in stmts {
            if !matches!(s, Statement::Root(_, _) | Statement::FunctionDefine(_, _, _)) {
                self.generate_stmt(s);
            }
        }

        self.emit(0x1000FFFF); // الـ Infinite Loop النهائي

        let mut binary = Vec::new();
        for &instr in &self.code {
            binary.extend_from_slice(&instr.to_be_bytes());
        }
        binary
    }

    fn generate_stmt(&mut self, s: &Statement) {
        match s {
            Statement::Outb(addr_expr, val_expr) | Statement::Poke(addr_expr, val_expr) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(val_expr, val_reg);
                
                let addr_reg = self.alloc_reg();
                self.gen_expr(addr_expr, addr_reg);

                self.emit(0xA0000000 | (addr_reg << 21) | (val_reg << 16));
                
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }
            Statement::FunctionDefine(n, p, b) => {
                self.functions.insert(n.clone(), self.code.len());
                self.emit(0x27BDFFE0); 
                self.emit(0xAFBF001C); 
                for (i, pn) in p.iter().enumerate() {
                    self.current_params.insert(pn.clone(), (8 + (i * 4)) as u32);
                }
                for bs in b { self.generate_stmt(bs); }
                self.emit(0x8FBF001C); 
                self.emit(0x27BD0020); 
                self.emit(0x03E00008); 
                self.current_params.clear();
            }
            Statement::Let(n, v) | Statement::Assignment(n, v) => {
                let val_reg = self.alloc_reg();
                self.gen_expr(v, val_reg);
                let ad = *self.symbols.entry(n.clone()).or_insert_with(|| {
                    let a = self.next_addr;
                    self.next_addr += 4;
                    a
                });
                let addr_reg = self.alloc_reg();
                self.emit_li(addr_reg, ad);
                self.emit(0xAC000000 | (addr_reg << 21) | (val_reg << 16)); 
                self.free_reg(addr_reg);
                self.free_reg(val_reg);
            }
            Statement::Loop(body) => {
                let start = self.code.len();
                self.loop_stack.push(Vec::new());
                for bs in body { self.generate_stmt(bs); }
                let target = (start * 4) / 4;
                self.emit(0x08000000 | (target as u32 & 0x03FFFFFF));
                let end = self.code.len();
                if let Some(mut ps) = self.loop_stack.pop() {
                    for p in ps {
                        let patch_target = (end * 4) / 4;
                        self.code[p] = 0x08000000 | (patch_target as u32 & 0x03FFFFFF);
                    }
                }
            }
            _ => {}
        }
    }

    fn gen_expr(&mut self, e: &Expression, dest_reg: u32) {
        match e {
            Expression::Number(n) => self.emit_li(dest_reg, *n as u32),
            Expression::Variable(s) => {
                if let Some(&val) = self.symbols.get(s) {
                    self.emit_li(dest_reg, val);
                } else if let Some(&offset) = self.current_params.get(s) {
                    self.emit(0x8FC00000 | (dest_reg << 16) | (offset & 0xFFFF));
                }
            }
            Expression::BinaryOp(l, op, r) => {
                let left_reg = self.alloc_reg();
                self.gen_expr(l, left_reg);
                let right_reg = self.alloc_reg();
                self.gen_expr(r, right_reg);
                match op.as_str() {
                    "+" => self.emit(0x00000021 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)),
                    "-" => self.emit(0x00000023 | (left_reg << 21) | (right_reg << 16) | (dest_reg << 11)),
                    _ => {}
                }
                self.free_reg(right_reg);
                self.free_reg(left_reg);
            }
            _ => {}
        }
    }

    fn emit(&mut self, instr: u32) { self.code.push(instr); }

    fn emit_li(&mut self, reg: u32, imm: u32) {
        let hi = (imm >> 16) & 0xFFFF;
        let lo = imm & 0xFFFF;

        if hi == 0 {
            self.emit(0x34000000 | (reg << 16) | lo); 
        } else {
            self.emit(0x3C000000 | (reg << 16) | hi); 
            if lo != 0 {
                self.emit(0x34000000 | (reg << 21) | (reg << 16) | lo); 
            }
        }
    }

    fn alloc_reg(&mut self) -> u32 { self.reg_pool.pop_front().unwrap_or(8) }
    fn free_reg(&mut self, reg: u32) { if (8..=15).contains(&reg) { self.reg_pool.push_back(reg); } }
}