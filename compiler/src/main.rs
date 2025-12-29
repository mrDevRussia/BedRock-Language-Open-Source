mod lexer;
mod parser;
mod ast;
mod codegen_asm;
mod codegen_bin;

use std::env;
use std::fs;
use std::path::Path;
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::codegen_asm::AsmGenerator;
use crate::codegen_bin::BinGenerator;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage(&args[0]);
        return;
    }

    // Basic CLI parsing
    let mut source_file = String::new();
    let mut format = String::new(); // "bin" or "asm"

    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--format" => {
                if i + 1 < args.len() {
                    format = args[i + 1].clone();
                    i += 1;
                } else {
                    eprintln!("Error: --format requires an argument (bin or asm)");
                    return;
                }
            }
            arg => {
                if source_file.is_empty() {
                    source_file = arg.to_string();
                } else {
                    // Unknown flag or extra argument
                }
            }
        }
        i += 1;
    }

    if source_file.is_empty() {
        eprintln!("Error: No input file provided");
        print_usage(&args[0]);
        return;
    }

    if format.is_empty() {
        eprintln!("Error: Please specify output format with --format <bin|asm>");
        return;
    }

    // Read source code
    let code = match fs::read_to_string(&source_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error reading file {}: {}", source_file, e);
            return;
        }
    };

    // Parse
    let lexer = Lexer::new(&code);
    let mut parser = Parser::new(lexer);

    match parser.parse_program() {
        Ok(program) => {
            println!("Parsing successful.");
            
            let output_path = Path::new(&source_file).with_extension(&format);
            
            match format.as_str() {
                "asm" => {
                    let mut generator = AsmGenerator::new();
                    let asm_code = generator.generate(&program);
                    if let Err(e) = fs::write(&output_path, asm_code) {
                        eprintln!("Failed to write output file: {}", e);
                    } else {
                        println!("Success! Assembly generated at: {}", output_path.display());
                    }
                }
                "bin" => {
                    let mut generator = BinGenerator::new();
                    let bin_code = generator.generate(&program);
                    if let Err(e) = fs::write(&output_path, bin_code) {
                        eprintln!("Failed to write output file: {}", e);
                    } else {
                        println!("Success! Binary generated at: {}", output_path.display());
                    }
                }
                _ => {
                    eprintln!("Error: Unknown format '{}'. Use 'bin' or 'asm'.", format);
                }
            }
        }
        Err(e) => {
            eprintln!("Parser Error: {}", e);
        }
    }
}

fn print_usage(program_name: &str) {
    println!("Usage: {} <file.br> --format <bin|asm>", program_name);
}
