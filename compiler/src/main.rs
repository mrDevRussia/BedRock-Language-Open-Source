mod lexer;
mod parser;
mod codegen;

use std::env;
use std::fs;
use std::path::Path;
use lexer::Lexer;
use parser::Parser;
use codegen::Codegen;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!("Usage: bedrockc <source_file> [--format bin|asm]");
        return;
    }

    let source_file = &args[1];
    let mut output_format = "bin";
    
    let mut i = 2;
    while i < args.len() {
        if args[i] == "--format" && i + 1 < args.len() {
            output_format = &args[i+1];
            i += 2;
        } else {
            i += 1;
        }
    }

    let code = fs::read_to_string(source_file).expect("Failed to read source file");
    
    let mut lexer = Lexer::new(&code);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == lexer::Token::EOF {
            break;
        }
        tokens.push(token);
    }
    
    // println!("Tokens: {:?}", tokens);

    let mut parser = Parser::new(tokens);
    let program = parser.parse_program();
    
    // println!("AST: {:?}", program);

    println!("Parsing complete. Generating code...");
    if output_format == "bin" {
        let mut codegen = Codegen::new();
        let binary = codegen.compile(&program);
        
        let output_file = Path::new(source_file).with_extension("bin");
        fs::write(output_file, binary).expect("Failed to write output file");
        println!("Compilation successful. Output: {}", Path::new(source_file).with_extension("bin").display());
    } else if output_format == "asm" {
        eprintln!("ASM format not supported in this version yet, generating binary instead.");
    } else {
        eprintln!("Unknown format: {}", output_format);
    }
}
