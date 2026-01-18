#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    Fn, Let, Loop, While, Asm, If, Else, Return, Root, Inb, Outb, Break, Poke, Peek, Include,
    Identifier(String), Number(u64), StringLiteral(String),
    LParen, RParen, LBrace, RBrace, LBracket, RBracket, Colon, SemiColon, Comma, Equal,
    Plus, Minus, Star, Slash, EqEq, Greater, Less, Ampersand, Pipe, EOF
}

pub struct Lexer { input: Vec<char>, pos: usize }
impl Lexer {
    pub fn new(input: &str) -> Self { Lexer { input: input.chars().collect(), pos: 0 } }
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
            ';' => { self.pos += 1; Token::SemiColon }
            ',' => { self.pos += 1; Token::Comma }
            '+' => { self.pos += 1; Token::Plus }
            '-' => { self.pos += 1; Token::Minus }
            '*' => { self.pos += 1; Token::Star }
            '/' => { self.pos += 1; Token::Slash }
            '&' => { self.pos += 1; Token::Ampersand }
            '|' => { self.pos += 1; Token::Pipe }
            '=' => {
                self.pos += 1;
                if self.pos < self.input.len() && self.input[self.pos] == '=' { self.pos += 1; Token::EqEq }
                else { Token::Equal }
            }
            '>' => { self.pos += 1; Token::Greater }
            '<' => { self.pos += 1; Token::Less }
            '"' => self.read_string(),
            _ => { self.pos += 1; self.next_token() }
        }
    }
    fn skip_whitespace(&mut self) { while self.pos < self.input.len() && self.input[self.pos].is_whitespace() { self.pos += 1; } }
    fn read_identifier(&mut self) -> Token {
        let start = self.pos;
        while self.pos < self.input.len() && (self.input[self.pos].is_alphanumeric() || self.input[self.pos] == '_') { self.pos += 1; }
        let s: String = self.input[start..self.pos].iter().collect();
        match s.as_str() {
            "fn" => Token::Fn, "let" => Token::Let, "loop" => Token::Loop, "while" => Token::While, "asm" => Token::Asm,
            "if" => Token::If, "else" => Token::Else, "return" => Token::Return, "root" => Token::Root,
            "inb" => Token::Inb, "outb" => Token::Outb, "break" => Token::Break,
            "poke" => Token::Poke, "peek" => Token::Peek, "include" => Token::Include, _ => Token::Identifier(s)
        }
    }
    fn read_number(&mut self) -> Token {
        let mut base = 10;
        let mut start = self.pos;
        if self.input[self.pos] == '0' && self.pos + 1 < self.input.len() {
            let next = self.input[self.pos + 1].to_ascii_lowercase();
            if next == 'x' { base = 16; self.pos += 2; start = self.pos; }
        }
        while self.pos < self.input.len() {
            let ch = self.input[self.pos].to_ascii_lowercase();
            if (base == 10 && !ch.is_digit(10)) || (base == 16 && !ch.is_digit(16)) { break; }
            self.pos += 1;
        }
        let s: String = self.input[start..self.pos].iter().collect();
        let value = u64::from_str_radix(&s, base).unwrap_or(0);
        Token::Number(value)
    }
    fn read_string(&mut self) -> Token {
        self.pos += 1; let start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != '"' { self.pos += 1; }
        let s = self.input[start..self.pos].iter().collect();
        self.pos += 1; Token::StringLiteral(s)
    }
}