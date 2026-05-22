mod type_inference;
mod ast; mod lexer; mod parser; mod codegen;
use std::{env, fs, path::Path};

fn process_includes(
    content: String,
    base_path: &Path,
    included: &mut std::collections::HashSet<String>,
    stack: &mut Vec<String>,
) -> String {
    let mut final_code = String::new();

    for line in content.lines() {
        if line.trim().starts_with("include ") {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() > 1 {
                let file_name = parts[1].replace("\"", "").replace(";", "");
                let include_path = base_path.join(&file_name);


                let canonical = match include_path.canonicalize() {
    Ok(p) => p.to_string_lossy().to_string(),
    Err(_) => {

        include_path.to_string_lossy().to_string()
    }
};


                if stack.contains(&canonical) {
                    let chain: Vec<String> = stack.iter()
    .map(|p| Path::new(p).file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string())
        .collect();
          eprintln!(
              "[INCLUDE ERROR] Circular include detected!\n  chain: {} -> {}",
                 chain.join(" -> "),
                   file_name
                  );
                    std::process::exit(1);
                }


                if included.contains(&canonical) {
                    continue;
                }

                included.insert(canonical.clone());
                stack.push(canonical.clone());

                let include_content = match fs::read_to_string(&include_path) {
    Ok(content) => content,
    Err(_) => {
        eprintln!(
            "[INCLUDE ERROR] File not found: '{}'\n  looked in: {}",
            file_name,
            include_path.display()
        );
        std::process::exit(1);
    }
};
                final_code.push_str(&process_includes(
                    include_content,
                    include_path.parent().unwrap_or(base_path),
                    included,
                    stack,
                ));
                final_code.push('\n');

                stack.pop();
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
    let source_code = match fs::read_to_string(source_path) {
    Ok(content) => content,
    Err(_) => {
        eprintln!(
            "[ERROR] Source file not found: '{}'\n  looked in: {}",
            args[1],
            source_path.display()
        );
        std::process::exit(1);
    }
};
    let base_path = source_path.parent().unwrap_or(Path::new("."));
    let mut included = std::collections::HashSet::new();
    let mut stack = Vec::new();
    let processed_code = process_includes(source_code, base_path, &mut included, &mut stack);
    let mut lexer = lexer::Lexer::new(&processed_code);
    let mut tokens = Vec::new();
    loop {
        let token = lexer.next_token();
        if token == lexer::Token::EOF { break; }
        tokens.push(token);
    }
    let mut parser = parser::Parser::new(tokens);
    let program = parser.parse_program();
    let mut inferencer = type_inference::TypeInferencer::new();
    let program = inferencer.run(program);

    let mut codegen = codegen::Codegen::new();
    let binary = codegen.compile(&program);
    fs::write(source_path.with_extension("bin"), &binary).expect("Write failed");
    let map_json = serde_json::to_string_pretty(codegen.get_source_map()).expect("Map failed");
    fs::write(source_path.with_extension("map.json"), map_json).expect("Map write failed");
}