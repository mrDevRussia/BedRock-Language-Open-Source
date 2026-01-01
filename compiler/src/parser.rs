use crate::lexer::Token;

#[derive(Debug, Clone, PartialEq)]
pub enum Type { U8, U16, U32, U64, Void, Pointer(Box<Type>) }

#[derive(Debug)]
pub struct Program { pub globals: Vec<Global>, pub functions: Vec<Function> }

#[derive(Debug)]
pub struct Global { pub name: String, pub ty: Type, pub volatile: bool, pub attributes: Vec<Attribute> }

#[derive(Debug)]
pub struct Function { pub name: String, pub ret_type: Type, pub body: Vec<Statement>, pub attributes: Vec<Attribute> }

#[derive(Debug, Clone)]
pub enum Attribute { Address(u64), Interrupt }

#[derive(Debug)]
pub enum Statement {
    Let { name: String, ty: Type, value: Option<Expression>, volatile: bool },
    Expression(Expression),
    Loop(Vec<Statement>),
    Asm(String),
    Assignment(Box<Expression>, Box<Expression>),
    Clear,
    Newline,
    Print(String, u8),
}

#[derive(Debug)]
pub enum Expression {
    Number(u64), Variable(String),
    BinaryOp(Box<Expression>, Op, Box<Expression>),
    Dereference(Box<Expression>),
}

#[derive(Debug, Clone)]
pub enum Op { Add, Sub, Or, And }

pub struct Parser { tokens: Vec<Token>, pos: usize }

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, pos: 0 } }

    pub fn parse_program(&mut self) -> Program {
        let mut globals = Vec::new();
        let mut functions = Vec::new();
        while !self.is_at_end() {
            let attrs = self.parse_attributes();
            if self.check(Token::Fn) { functions.push(self.parse_function(attrs)); }
            else if self.check(Token::Let) || self.check(Token::Volatile) { globals.push(self.parse_global(attrs)); }
            else { self.advance(); }
        }
        Program { globals, functions }
    }

    fn parse_attributes(&mut self) -> Vec<Attribute> {
        let mut attrs = Vec::new();
        while self.match_token(Token::Hash) {
            self.expect(Token::LBracket);
            match self.advance() {
                Token::Identifier(ref s) if s == "address" => {
                    self.expect(Token::LParen);
                    let addr = match self.advance() { Token::Number(n) => n, _ => panic!("Addr expected") };
                    self.expect(Token::RParen);
                    attrs.push(Attribute::Address(addr));
                }
                Token::Identifier(ref s) if s == "interrupt" => attrs.push(Attribute::Interrupt),
                _ => panic!("Unknown attribute"),
            }
            self.expect(Token::RBracket);
        }
        attrs
    }

    fn parse_function(&mut self, attributes: Vec<Attribute>) -> Function {
        self.expect(Token::Fn);
        let name = match self.advance() { Token::Identifier(s) => s, _ => panic!("Fn name expected") };
        self.expect(Token::LParen);
        while !self.check(Token::RParen) && !self.is_at_end() { 
            self.advance(); 
            if self.match_token(Token::Comma) { continue; } 
        }
        self.expect(Token::RParen);
        self.expect(Token::Arrow);
        let ret_type = self.parse_type();
        self.expect(Token::LBrace);
        Function { name, ret_type, body: self.parse_block(), attributes }
    }

    fn parse_global(&mut self, attributes: Vec<Attribute>) -> Global {
        let volatile = self.match_token(Token::Volatile);
        self.expect(Token::Let);
        let name = match self.advance() { Token::Identifier(s) => s, _ => panic!("Global name") };
        self.expect(Token::Colon);
        let ty = self.parse_type();
        self.expect(Token::SemiColon);
        Global { name, ty, volatile, attributes }
    }

    fn parse_block(&mut self) -> Vec<Statement> {
        let mut stmts = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() { stmts.push(self.parse_statement()); }
        self.expect(Token::RBrace);
        stmts
    }

    fn parse_statement(&mut self) -> Statement {
        if self.match_token(Token::Let) {
            let name = match self.advance() { Token::Identifier(s) => s, _ => panic!("Var name") };
            self.expect(Token::Colon);
            let ty = self.parse_type();
            self.expect(Token::Equal);
            let val = self.parse_expression();
            self.expect(Token::SemiColon);
            Statement::Let { name, ty, value: Some(val), volatile: false }
        } else if self.check(Token::Identifier("clear".to_string())) {
            self.advance(); self.expect(Token::LParen); self.expect(Token::RParen); self.expect(Token::SemiColon);
            Statement::Clear
        } else if self.check(Token::Identifier("newline".to_string())) {
            self.advance(); self.expect(Token::LParen); self.expect(Token::RParen); self.expect(Token::SemiColon);
            Statement::Newline
        } else if self.check(Token::Identifier("print".to_string())) {
            self.advance(); self.expect(Token::LParen);
            let s = match self.advance() { Token::StringLiteral(s) => s, _ => panic!("Print string expected") };
            let col = if self.match_token(Token::Comma) {
                match self.advance() { Token::Number(n) => n as u8, _ => 0x07 }
            } else { 0x07 };
            self.expect(Token::RParen); self.expect(Token::SemiColon);
            Statement::Print(s, col)
        } else if self.match_token(Token::Loop) {
            self.expect(Token::LBrace); Statement::Loop(self.parse_block())
        } else if self.match_token(Token::Asm) {
            self.expect(Token::LParen);
            let code = match self.advance() { Token::StringLiteral(s) => s, _ => panic!("Asm string") };
            self.expect(Token::RParen); self.expect(Token::SemiColon);
            Statement::Asm(code)
        } else {
            let expr = self.parse_expression();
            if self.match_token(Token::Equal) {
                let rhs = self.parse_expression(); self.expect(Token::SemiColon);
                Statement::Assignment(Box::new(expr), Box::new(rhs))
            } else { self.expect(Token::SemiColon); Statement::Expression(expr) }
        }
    }

    fn parse_expression(&mut self) -> Expression {
        self.parse_additive()
    }

    fn parse_additive(&mut self) -> Expression {
        let mut left = self.parse_bitwise();
        while self.check(Token::Plus) || self.check(Token::Minus) {
            let op = if self.match_token(Token::Plus) { Op::Add } else { self.advance(); Op::Sub };
            left = Expression::BinaryOp(Box::new(left), op, Box::new(self.parse_bitwise()));
        }
        left
    }

    fn parse_bitwise(&mut self) -> Expression {
        let mut left = self.parse_unary();
        while self.match_token(Token::Pipe) {
            left = Expression::BinaryOp(Box::new(left), Op::Or, Box::new(self.parse_unary()));
        }
        left
    }

    fn parse_unary(&mut self) -> Expression {
        if self.match_token(Token::Star) { Expression::Dereference(Box::new(self.parse_unary())) }
        else { self.parse_primary() }
    }

    fn parse_primary(&mut self) -> Expression {
        if self.match_token(Token::LParen) {
            let expr = self.parse_expression();
            self.expect(Token::RParen);
            expr
        } else {
            match self.advance() {
                Token::Number(n) => Expression::Number(n),
                Token::Identifier(s) => Expression::Variable(s),
                _ => panic!("Expression error at {:?}", self.peek()),
            }
        }
    }

    fn parse_type(&mut self) -> Type {
        if self.match_token(Token::Star) { Type::Pointer(Box::new(self.parse_type())) }
        else {
            match self.advance() {
                Token::Identifier(s) => match s.as_str() {
                    "u8" => Type::U8, "u16" => Type::U16, "u32" => Type::U32, "void" => Type::Void,
                    _ => panic!("Unknown type: {}", s),
                },
                _ => panic!("Type expected"),
            }
        }
    }

    fn peek(&self) -> Token { self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF) }
    fn check(&self, t: Token) -> bool { self.peek() == t }
    fn is_at_end(&self) -> bool { self.peek() == Token::EOF }
    fn advance(&mut self) -> Token { let t = self.peek(); if !self.is_at_end() { self.pos += 1; } t }
    fn match_token(&mut self, t: Token) -> bool { if self.check(t) { self.advance(); true } else { false } }
    fn expect(&mut self, t: Token) { if !self.match_token(t.clone()) { panic!("Expected {:?} got {:?}", t, self.peek()); } }
}