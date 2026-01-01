use crate::parser::{Program, Function, Statement, Expression, Op};
use std::collections::HashMap;

pub struct Codegen {
    code: Vec<u8>,
    functions: HashMap<String, u16>,
    current_offset: u16,
}

impl Codegen {
    pub fn new() -> Self {
        Codegen { code: Vec::new(), functions: HashMap::new(), current_offset: 0 }
    }

    pub fn compile(&mut self, program: &Program) -> Vec<u8> {
        self.code.clear(); self.current_offset = 0;
        self.emit_u8(0xE9); self.emit_u16(0); 

        for func in &program.functions {
            self.functions.insert(func.name.clone(), self.current_offset);
            self.generate_function(func);
        }

        if let Some(&main_off) = self.functions.get("kernel_main") {
            let rel = (main_off as i32 - 3) as u16;
            self.code[1] = (rel & 0xFF) as u8; self.code[2] = (rel >> 8) as u8;
        }
        self.code.clone()
    }

    fn generate_function(&mut self, func: &Function) {
        self.emit_u8(0x55); // push bp
        self.emit_u8(0x89); self.emit_u8(0xE5); // mov bp, sp
        for stmt in &func.body { self.generate_statement(stmt); }
        self.emit_u8(0x89); self.emit_u8(0xEC); // mov sp, bp
        self.emit_u8(0x5D); // pop bp
        self.emit_u8(0xC3); // ret
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Clear => {
                self.emit_u8(0xB8); self.emit_u16(0x0003); 
                self.emit_u8(0xCD); self.emit_u8(0x10);    
            }
            Statement::Newline => {
                for c in [0x0D, 0x0A] {
                    self.emit_u8(0xB4); self.emit_u8(0x0E); 
                    self.emit_u8(0xB0); self.emit_u8(c);    
                    self.emit_u8(0xCD); self.emit_u8(0x10); 
                }
            }
            Statement::Print(text, _color) => {
                for c in text.chars() {
                    self.emit_u8(0xB4); self.emit_u8(0x0E); 
                    self.emit_u8(0xB0); self.emit_u8(c as u8); 
                    self.emit_u8(0xCD); self.emit_u8(0x10); 
                }
            }
            Statement::Loop(body) => {
                let start = self.current_offset;
                for s in body { self.generate_statement(s); }
                self.emit_u8(0xE9);
                let rel = (start as i32 - (self.current_offset as i32 + 3)) as u16;
                self.emit_u16(rel);
            }
            Statement::Asm(code) => {
                if code.contains("hlt") { self.emit_u8(0xF4); }
            }
            Statement::Assignment(target, value) => {
                self.emit_expression(value); // Result in AX
                self.emit_u8(0x50); // push ax
                self.emit_expression(target); // Address in AX
                self.emit_u8(0x89); self.emit_u8(0xC3); // mov bx, ax
                self.emit_u8(0x58); // pop ax
                self.emit_u8(0x89); self.emit_u8(0x07); // mov [bx], ax
            }
            _ => {}
        }
    }

    fn emit_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Number(n) => {
                self.emit_u8(0xB8); self.emit_u16(*n as u16);
            }
            Expression::BinaryOp(left, op, right) => {
                self.emit_expression(left);
                self.emit_u8(0x50); 
                self.emit_expression(right);
                self.emit_u8(0x89); self.emit_u8(0xC3); 
                self.emit_u8(0x58); 
                match op {
                    Op::Add => { self.emit_u8(0x01); self.emit_u8(0xD8); }
                    Op::Sub => { self.emit_u8(0x29); self.emit_u8(0xD8); }
                    _ => {}
                }
            }
            Expression::Dereference(e) => {
                self.emit_expression(e);
                self.emit_u8(0x89); self.emit_u8(0xC3); 
                self.emit_u8(0x8B); self.emit_u8(0x07); 
            }
            _ => {}
        }
    }

    fn emit_u8(&mut self, b: u8) { self.code.push(b); self.current_offset += 1; }
    fn emit_u16(&mut self, w: u16) { self.emit_u8((w & 0xFF) as u8); self.emit_u8((w >> 8) as u8); }
}