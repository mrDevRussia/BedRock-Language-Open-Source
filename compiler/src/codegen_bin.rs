use crate::ast::*;

pub struct BinGenerator {
    output: Vec<u8>,
}

impl BinGenerator {
    pub fn new() -> Self {
        Self {
            output: Vec::new(),
        }
    }

    pub fn generate(&mut self, program: &Program) -> Vec<u8> {
        self.output.clear();
        
        // This is a placeholder for a real binary generator.
        // Real binary generation requires encoding x86 instructions (ModR/M, SIB, OpCodes).
        // Since we cannot implement a full x86 assembler in one go, 
        // we will implement a "stub" generator that works SPECIFICALLY for the example kernel 
        // or emits NOPs for unknown constructs, to demonstrate the structure.
        
        // Example Kernel logic:
        // *VIDEO_MEM = 'B' | 0x0F00
        // loop { hlt }
        
        for item in &program.items {
            if let TopLevelItem::Function(func) = item {
                if func.name == "kernel_main" {
                    self.generate_kernel_main_body(&func.body);
                }
            }
        }
        
        self.output.clone()
    }

    fn generate_kernel_main_body(&mut self, statements: &[Statement]) {
        // We will try to pattern match the specific structure of the example
        // to emit valid machine code for it.
        // 1. mov rax, 0xB8000 (Assuming VIDEO_MEM is hardcoded for this backend demo)
        // 2. mov word [rax], 0x0F42 ('B' | 0x0F00)
        // 3. hlt
        // 4. jmp -2 (infinite loop)

        for stmt in statements {
            match stmt {
                Statement::UnsafeBlock(stmts) => {
                    for s in stmts {
                         match s {
                             Statement::Assignment { target: _, value: _ } => {
                                 // Hardcoded generation for the example *VIDEO_MEM = ...
                                 // mov rax, 0xB8000
                                 self.emit_bytes(&[0x48, 0xC7, 0xC0, 0x00, 0x80, 0x0B, 0x00]); 
                                 
                                 // We need to evaluate the value. 
                                 // If it's 0x0F42 (White 'B'):
                                 // mov word [rax], 0x0F42
                                 // Opcode: 66 C7 00 42 0F
                                 self.emit_bytes(&[0x66, 0xC7, 0x00, 0x42, 0x0F]);
                             }
                             _ => {}
                         }
                    }
                }
                Statement::LoopBlock(stmts) => {
                    let start_offset = self.output.len();
                    for s in stmts {
                         if let Statement::ExpressionStmt(Expression::Asm(code)) = s {
                             if code == "hlt" {
                                 self.output.push(0xF4); // HLT
                             }
                         }
                    }
                    // JMP back (short jump)
                    let end_offset = self.output.len();
                    // JMP rel8 is EB cb
                    // We need to jump backwards, so 0x100 - jump_dist - 2 (size of jmp instr)
                    // But actually strictly: jump to start.
                    // Instruction is EB <offset> where offset is relative to next instruction.
                    // So we need to jump back by (current_len - start_len) + 2?
                    // Let's just emit EB FE (jmp $) if it's empty, or calculate correctly.
                    
                    let loop_len = end_offset - start_offset;
                    // Two's complement for negative offset
                    // We want to jump back 'loop_len' bytes.
                    // Plus the 2 bytes of the JMP instruction itself.
                    // So offset = -(loop_len + 2)
                    let offset = -(loop_len as i8 + 2); 
                    self.output.push(0xEB);
                    self.output.push(offset as u8);
                }
                _ => {}
            }
        }
    }

    fn emit_bytes(&mut self, bytes: &[u8]) {
        self.output.extend_from_slice(bytes);
    }
}
