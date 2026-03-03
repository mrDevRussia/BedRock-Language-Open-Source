#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn, Let, Loop, While, Asm, If, Else, Return, Root, Inb, Outb, Break, Poke, Peek, Include,
    Identifier(String), Number(u64), StringLiteral(String),
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Colon, SemiColon, Comma, Equal,
    Plus, Minus, Star, Slash, EqEq, Greater, Less, Ampersand, Pipe, EOF, Caret, NotEq, GreaterEq, LessEq
}

pub struct Lexer { 
    input: Vec<char>, 
    pos: usize,
    line: usize,
    col: usize
}

impl Lexer {
    pub fn new(input: &str) -> Self { 
        Lexer { 
            input: input.chars().collect(), 
            pos: 0,
            line: 1,
            col: 1
        } 
    }
    
    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace_and_comments();
        if self.pos >= self.input.len() { return Token::EOF; }
        
        let ch = self.input[self.pos];
        
        if ch.is_alphabetic() || ch == '_' { return self.read_identifier(); }
        if ch.is_digit(10) { return self.read_number(); }
        
        match ch {
            '(' => { self.advance_char(); Token::LParen }
            ')' => { self.advance_char(); Token::RParen }
            '{' => { self.advance_char(); Token::LBrace }
            '}' => { self.advance_char(); Token::RBrace }
            '[' => { self.advance_char(); Token::LBracket }
            ']' => { self.advance_char(); Token::RBracket }
            ';' => { self.advance_char(); Token::SemiColon }
            ',' => { self.advance_char(); Token::Comma }
            '+' => { self.advance_char(); Token::Plus }
            '-' => { self.advance_char(); Token::Minus }
            '^' => { self.advance_char(); Token::Caret }
            '*' => { self.advance_char(); Token::Star }
            '&' => { self.advance_char(); Token::Ampersand }
            '|' => { self.advance_char(); Token::Pipe }
            '/' => {
                self.advance_char();
                if self.pos < self.input.len() && self.input[self.pos] == '/' {
                    self.skip_line_comment();
                    return self.next_token();
                } else if self.pos < self.input.len() && self.input[self.pos] == '*' {
                    self.skip_block_comment();
                    return self.next_token();
                }
                Token::Slash
            }
            '=' => {
                self.advance_char();
                if self.pos < self.input.len() && self.input[self.pos] == '=' { 
                    self.advance_char(); Token::EqEq 
                } else { Token::Equal }
            }
            '!' => {
                self.advance_char();
                if self.pos < self.input.len() && self.input[self.pos] == '=' {
                    self.advance_char(); Token::NotEq
                } else { 
                    eprintln!("[LEXER ERROR] Expected '=' after '!' at line {}, col {}", self.line, self.col);
                    Token::EOF 
                }
            }
            '>' => {
                self.advance_char();
                if self.pos < self.input.len() && self.input[self.pos] == '=' {
                    self.advance_char(); Token::GreaterEq
                } else { Token::Greater }
            }
            '<' => {
                self.advance_char();
                if self.pos < self.input.len() && self.input[self.pos] == '=' {
                    self.advance_char(); Token::LessEq
                } else { Token::Less }
            }
            '"' => {
                self.advance_char(); 
                self.read_string()
            }

            _ => {
                if ch.is_whitespace() {
                    self.advance_char();
                    return self.next_token();
                }
                eprintln!("[LEXER ERROR] Unknown character '{}' at line {}, col {}", ch, self.line, self.col);
                self.advance_char();
                self.next_token()
            }
        } 
    } 
    
    fn advance_char(&mut self) {
        if self.pos < self.input.len() {
            if self.input[self.pos] == '\n' {
                self.line += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
            self.pos += 1;
        }
    }
    
    fn skip_whitespace_and_comments(&mut self) {
        while self.pos < self.input.len() {
            let ch = self.input[self.pos];
            if ch.is_whitespace() {
                self.advance_char();
            } else {
                break;
            }
        }
    }
    
    fn skip_line_comment(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos] != '\n' {
            self.advance_char();
        }
    }
    
    fn skip_block_comment(&mut self) {
        self.advance_char(); // Skip '*'
        while self.pos + 1 < self.input.len() {
            if self.input[self.pos] == '*' && self.input[self.pos + 1] == '/' {
                self.advance_char();
                self.advance_char();
                return;
            }
            self.advance_char();
        }
        eprintln!("[LEXER ERROR] Unterminated block comment");
    }
    
    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && 
              (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') { 
            self.advance_char();
        }
        let s: String = self.input[start..self.pos].iter().collect();
        match s.as_str() {
            "fn" => Token::Fn, 
            "let" => Token::Let, 
            "loop" => Token::Loop, 
            "while" => Token::While, 
            "asm" => Token::Asm,
            "if" => Token::If, 
            "else" => Token::Else, 
            "return" => Token::Return, 
            "root" => Token::Root,
            "inb" => Token::Inb, 
            "outb" => Token::Outb, 
            "break" => Token::Break,
            "poke" => Token::Poke, 
            "peek" => Token::Peek, 
            "include" => Token::Include, 
            _ => Token::Identifier(s)
        }
    }
    
    fn read_number(&mut self) -> Token {
        let mut base = 10;
        let mut start = self.pos;
        
        if self.input[self.pos] == '0' && self.pos + 1 < self.input.len() {
            let next = self.input[self.pos + 1].to_ascii_lowercase();
            if next == 'x' { 
                base = 16; 
                self.advance_char();
                self.advance_char();
                start = self.pos; 
            }
        }
        
        while self.pos < self.input.len() {
            let ch = self.input[self.pos].to_ascii_lowercase();
            if (base == 10 && !ch.is_digit(10)) || (base == 16 && !ch.is_digit(16)) { 
                break; 
            }
            self.advance_char();
        }
        
        let s: String = self.input[start..self.pos].iter().collect();
        let value = u64::from_str_radix(&s, base).unwrap_or_else(|_| {
            eprintln!("[LEXER ERROR] Invalid number format: {}", s);
            0
        });
        Token::Number(value)
    }
    
    fn read_string(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != '"' { 
            self.advance_char();
        }
        if self.pos >= self.input.len() {
            eprintln!("[LEXER ERROR] Unterminated string literal");
            return Token::EOF;
        }
        let s: String = self.input[start..self.pos].iter().collect();
        self.advance_char(); // Skip the closing quote
        Token::StringLiteral(s)
    }
}