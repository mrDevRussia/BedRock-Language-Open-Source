#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Fn,
    Let,
    Volatile,
    Cast,
    Struct,
    Unsafe,
    Loop,
    Asm,

    // Identifiers and Literals
    Identifier(String),
    Integer(u64),
    StringLiteral(String),

    // Symbols
    Arrow,          // ->
    Colon,          // :
    SemiColon,      // ;
    Equals,         // =
    Star,           // *
    Pipe,           // |
    LParen,         // (
    RParen,         // )
    LBrace,         // {
    RBrace,         // }
    LBracket,       // [
    RBracket,       // ]
    Comma,          // ,
    Hash,           // #
    LessThan,       // <
    GreaterThan,    // >
    
    // End of File
    EOF,
}

pub struct Lexer {
    input: Vec<char>,
    position: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            position: 0,
        }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();

        if self.position >= self.input.len() {
            return Token::EOF;
        }

        let ch = self.input[self.position];

        if ch.is_alphabetic() || ch == '_' {
            return self.read_identifier_or_keyword();
        }

        if ch.is_digit(10) {
            return self.read_number();
        }

        match ch {
            '-' => {
                if self.peek() == '>' {
                    self.position += 2;
                    Token::Arrow
                } else {
                    // Assuming only -> for now based on specs, but - could be operator
                    // For now, treat as single char logic if needed, but spec is minimal.
                    // Let's assume just -> for function return type.
                    panic!("Unexpected character: -"); 
                }
            }
            ':' => { self.position += 1; Token::Colon }
            ';' => { self.position += 1; Token::SemiColon }
            '=' => { self.position += 1; Token::Equals }
            '*' => { self.position += 1; Token::Star }
            '|' => { self.position += 1; Token::Pipe }
            '(' => { self.position += 1; Token::LParen }
            ')' => { self.position += 1; Token::RParen }
            '{' => { self.position += 1; Token::LBrace }
            '}' => { self.position += 1; Token::RBrace }
            '[' => { self.position += 1; Token::LBracket }
            ']' => { self.position += 1; Token::RBracket }
            ',' => { self.position += 1; Token::Comma }
            '#' => { self.position += 1; Token::Hash }
            '<' => { self.position += 1; Token::LessThan }
            '>' => { self.position += 1; Token::GreaterThan }
            '/' => {
                if self.peek() == '/' {
                    self.skip_comment();
                    self.next_token()
                } else {
                    panic!("Unexpected character: /");
                }
            }
            '"' => self.read_string(),
            _ => panic!("Unexpected character: {}", ch),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.position < self.input.len() && self.input[self.position].is_whitespace() {
            self.position += 1;
        }
    }

    fn skip_comment(&mut self) {
        while self.position < self.input.len() && self.input[self.position] != '\n' {
            self.position += 1;
        }
    }

    fn peek(&self) -> char {
        if self.position + 1 >= self.input.len() {
            '\0'
        } else {
            self.input[self.position + 1]
        }
    }

    fn read_identifier_or_keyword(&mut self) -> Token {
        let start = self.position;
        while self.position < self.input.len() && (self.input[self.position].is_alphanumeric() || self.input[self.position] == '_') {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();

        match text.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "volatile" => Token::Volatile,
            "cast" => Token::Cast,
            "struct" => Token::Struct,
            "unsafe" => Token::Unsafe,
            "loop" => Token::Loop,
            "asm" => Token::Asm,
            _ => Token::Identifier(text),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.position;
        // Check for hex 0x
        if self.input[self.position] == '0' && self.peek() == 'x' {
            self.position += 2;
            while self.position < self.input.len() && self.input[self.position].is_digit(16) {
                self.position += 1;
            }
            let text: String = self.input[start+2..self.position].iter().collect();
            let value = u64::from_str_radix(&text, 16).unwrap();
            return Token::Integer(value);
        }

        while self.position < self.input.len() && self.input[self.position].is_digit(10) {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();
        let value = text.parse().unwrap();
        Token::Integer(value)
    }

    fn read_string(&mut self) -> Token {
        self.position += 1; // skip opening quote
        let start = self.position;
        while self.position < self.input.len() && self.input[self.position] != '"' {
            self.position += 1;
        }
        let text: String = self.input[start..self.position].iter().collect();
        self.position += 1; // skip closing quote
        Token::StringLiteral(text)
    }
}
