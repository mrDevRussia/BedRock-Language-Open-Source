#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn, Let, Volatile, Unsafe, Loop, Asm, Cast,
    Identifier(String), Number(u64), StringLiteral(String),
    LParen, RParen, LBrace, RBrace, LBracket, RBracket,
    Colon, SemiColon, Comma, Equal, Star, Arrow, Pipe, Hash, Plus, Minus,
    LessThan, GreaterThan, Dot, EOF,
}

pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer { input: input.chars().collect(), pos: 0 }
    }

    pub fn next_token(&mut self) -> Token {
        self.skip_whitespace();
        if self.pos >= self.input.len() { return Token::EOF; }
        let ch = self.input[self.pos];

        if ch.is_alphabetic() || ch == '_' { return self.read_identifier(); }
        if ch.is_digit(10) { return self.read_number(); }

        match ch {
            '(' => { self.pos += 1; Token::LParen }
            ')' => { self.pos += 1; Token::RParen }
            '{' => { self.pos += 1; Token::LBrace }
            '}' => { self.pos += 1; Token::RBrace }
            '[' => { self.pos += 1; Token::LBracket }
            ']' => { self.pos += 1; Token::RBracket }
            ':' => { self.pos += 1; Token::Colon }
            ';' => { self.pos += 1; Token::SemiColon }
            ',' => { self.pos += 1; Token::Comma }
            '=' => { self.pos += 1; Token::Equal }
            '*' => { self.pos += 1; Token::Star }
            '|' => { self.pos += 1; Token::Pipe }
            '#' => { self.pos += 1; Token::Hash }
            '+' => { self.pos += 1; Token::Plus }
            '.' => { self.pos += 1; Token::Dot }
            '<' => { self.pos += 1; Token::LessThan }
            '>' => { self.pos += 1; Token::GreaterThan }
            '-' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '>' {
                    self.pos += 2; Token::Arrow
                } else { self.pos += 1; Token::Minus }
            }
            '"' => self.read_string(),
            '/' => {
                if self.pos + 1 < self.input.len() && self.input[self.pos + 1] == '/' {
                    self.skip_comment(); self.next_token()
                } else { panic!("Unexpected /"); }
            }
            _ => panic!("Unexpected char {} at pos {}", ch, self.pos),
        }
    }

    fn skip_whitespace(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; }
    }

    fn skip_comment(&mut self) {
        while self.pos < self.input.len() && self.input[self.pos] != '\n' { self.pos += 1; }
    }

    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') {
            self.pos += 1;
        }
        let ident: String = self.input[start..self.pos].iter().collect();
        match ident.as_str() {
            "fn" => Token::Fn,
            "let" => Token::Let,
            "volatile" => Token::Volatile,
            "unsafe" => Token::Unsafe,
            "loop" => Token::Loop,
            "asm" => Token::Asm,
            "cast" => Token::Cast,
            _ => Token::Identifier(ident),
        }
    }

    fn read_number(&mut self) -> Token {
        let start = self.pos;
        // دعم الـ Hexadecimal 0x
        if self.input[self.pos] == '0' && self.pos+1 < self.input.len() && self.input[self.pos+1].to_ascii_lowercase() == 'x' {
            self.pos += 2;
            let hex_start = self.pos;
            while self.pos < self.input.len() && self.input[self.pos].is_digit(16) { self.pos += 1; }
            let hex_str: String = self.input[hex_start..self.pos].iter().collect();
            let num = u64::from_str_radix(&hex_str, 16).expect("Invalid hex number");
            return Token::Number(num);
        }
        while self.pos < self.input.len() && self.input[self.pos].is_digit(10) { self.pos += 1; }
        Token::Number(self.input[start..self.pos].iter().collect::<String>().parse().unwrap())
    }

    fn read_string(&mut self) -> Token {
        self.pos += 1; // skip "
        let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != '"' { self.pos += 1; }
        let s = self.input[start..self.pos].iter().collect();
        self.pos += 1; // skip "
        Token::StringLiteral(s)
    }
}