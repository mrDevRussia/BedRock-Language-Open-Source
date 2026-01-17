mod ast;
mod lexer;
mod parser;
mod codegen;

use std::{env, fs, path::Path};

fn process_includes(content: String, base_path: &Path) -> String {
    let mut final_code = String::new();
    for line in content.lines() {
        if line.trim().starts_with("include ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                let file_name = parts[1].replace("\"", "").replace(";", "");
                let include_path = base_path.join(file_name);
                let include_content = fs::read_to_string(&include_path).expect("Include file not found");
                final_code.push_str(&process_includes(include_content, base_path));
                final_code.push('\n');
            }
        } else {
            final_code.push_str(line);
            final_code.push('\n');
        }
    }
    final_code
}

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 { return; }
    let source_path = Path::new(&args[1]);
    let source_code = fs::read_to_string(source_path).expect("Could not read file");
    let base_path = source_path.parent().unwrap_or(Path::new("."));
    
    let processed_code = process_includes(source_code, base_path);
    
    let mut lexer = lexer::Lexer::new(&processed_code);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == lexer::Token::EOF { break; }
        tokens.push(token);
    }
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program();
    let mut codegen = codegen::Codegen::new();
    let binary = codegen.compile(&program);
    let output_path = source_path.with_extension("bin");
    fs::write(&output_path, &binary).expect("Write failed");
    
    let mut img = vec![0u8; 1474560];
    img[0..binary.len().min(512)].copy_from_slice(&binary[0..binary.len().min(512)]);
    fs::write("bedrock_os.img", img).expect("Image creation failed");
}