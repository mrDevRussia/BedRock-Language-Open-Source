use crate::ast::*;
use std::collections::HashMap;

pub struct AsmGenerator {
    output: String,
    locals: HashMap<String, i32>,
    current_stack_offset: i32,
}

impl AsmGenerator {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            locals: HashMap::new(),
            current_stack_offset: 0,
        }
    }

    pub fn generate(&mut self, program: &Program) -> String {
        self.output.clear();
        
        // Add basic header
        self.output.push_str("bits 64\n");
        self.output.push_str("section .text\n");
        self.output.push_str("global kernel_main\n\n");

        for item in &program.items {
            match item {
                TopLevelItem::GlobalVariable(global) => self.generate_global(global),
                TopLevelItem::Function(func) => self.generate_function(func),
            }
        }
        
        self.output.clone()
    }

    fn generate_global(&mut self, global: &GlobalVariable) {
        // Handle global variables
        // For #[address(addr)], we might treat them as constants/equ if they are pointers
        
        let mut addr: Option<u64> = None;
        for attr in &global.attributes {
            if let Attribute::Address(a) = attr {
                addr = Some(*a);
            }
        }

        if let Some(address) = addr {
            // Define as a constant symbol
            self.output.push_str(&format!("{} equ {}\n", global.name, address));
        } else {
            // Fallback for non-address globals
            self.output.push_str(&format!("; Global {} (dynamic allocation not fully supported in simple backend)\n", global.name));
        }
    }

    fn generate_function(&mut self, func: &Function) {
        self.output.push_str(&format!("{}:\n", func.name));
        
        // Reset local tracking for new function
        self.locals.clear();
        self.current_stack_offset = 0;

        // Prologue
        self.output.push_str("    push rbp\n");
        self.output.push_str("    mov rbp, rsp\n");

        for stmt in &func.body {
            self.generate_statement(stmt);
        }

        // Epilogue
        self.output.push_str("    mov rsp, rbp\n"); // Clean up stack
        self.output.push_str("    pop rbp\n");
        self.output.push_str("    ret\n\n");
    }

    fn generate_statement(&mut self, stmt: &Statement) {
        match stmt {
            Statement::Let { name, value, .. } => {
                // 1. Evaluate expression to RAX
                self.generate_expression(value);
                
                // 2. Push RAX (allocating local var)
                self.output.push_str("    push rax\n");
                
                // 3. Track offset
                self.current_stack_offset -= 8;
                self.locals.insert(name.clone(), self.current_stack_offset);
                
                self.output.push_str(&format!("    ; variable {} at [rbp{}]\n", name, self.current_stack_offset));
            }
            Statement::UnsafeBlock(stmts) => {
                for s in stmts {
                    self.generate_statement(s);
                }
            }
            Statement::LoopBlock(stmts) => {
                let label = format!(".L_loop_{}", self.output.len());
                self.output.push_str(&format!("{}:\n", label));
                for s in stmts {
                    self.generate_statement(s);
                }
                self.output.push_str(&format!("    jmp {}\n", label));
            }
            Statement::ExpressionStmt(expr) => {
                self.generate_expression(expr);
            }
            Statement::Assignment { target, value } => {
                // target = value
                // 1. Evaluate value -> RAX
                self.generate_expression(value);
                self.output.push_str("    push rax\n"); // Save value

                // 2. Evaluate target address
                match target {
                    Expression::Dereference(inner) => {
                        // *ptr = val
                        // Evaluate ptr -> RAX
                        self.generate_expression(inner);
                        self.output.push_str("    pop rbx\n"); // Pop value into RBX
                        
                        // RAX has address, RBX has value
                        // We use generic 'mov [rax], bx' (assuming 16-bit for this example)
                        // A real compiler would check the type of *ptr.
                        // For kernel.br example: *VIDEO_MEM (u16) = ...
                        self.output.push_str("    mov [rax], bx\n");
                    }
                    _ => {
                        self.output.push_str("    ; Complex assignment not supported in this demo\n");
                        self.output.push_str("    add rsp, 8\n"); // Cleanup pushed value if we don't use it
                    }
                }
            }
        }
    }

    fn generate_expression(&mut self, expr: &Expression) {
        match expr {
            Expression::Integer(val) => {
                self.output.push_str(&format!("    mov rax, {}\n", val));
            }
            Expression::Identifier(name) => {
                if let Some(offset) = self.locals.get(name) {
                    // It is a local variable
                    self.output.push_str(&format!("    mov rax, [rbp{}]\n", offset));
                } else {
                    // Assume it is a global (EQU or Label)
                    self.output.push_str(&format!("    mov rax, {}\n", name));
                }
            }
            Expression::Cast { value, .. } => {
                // Casts are usually no-ops in asm or just truncation/extension
                self.generate_expression(value);
            }
            Expression::Dereference(inner) => {
                self.generate_expression(inner);
                self.output.push_str("    mov rax, [rax]\n");
            }
            Expression::BinaryOp { op, left, right } => {
                self.generate_expression(left);
                self.output.push_str("    push rax\n");
                self.generate_expression(right);
                self.output.push_str("    mov rbx, rax\n");
                self.output.push_str("    pop rax\n");
                match op {
                    BinaryOperator::BitwiseOr => {
                        self.output.push_str("    or rax, rbx\n");
                    }
                }
            }
            Expression::FunctionCall { name, args } => {
                // BedRock example: inb(cast<u16>(0x60))
                if name == "inb" {
                     if let Some(arg) = args.first() {
                         self.generate_expression(arg);
                         // in al, dx
                         self.output.push_str("    mov dx, ax\n");
                         self.output.push_str("    in al, dx\n");
                         self.output.push_str("    and rax, 0xFF\n"); // clear upper bits
                     }
                } else {
                    // Standard call
                    self.output.push_str(&format!("    call {}\n", name));
                }
            }
            Expression::Asm(code) => {
                self.output.push_str(&format!("    {}\n", code));
            }
        }
    }
}
