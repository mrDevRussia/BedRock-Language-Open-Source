use crate::lexer::Token;
use crate::ast::{Statement, Expression};

pub struct Parser { tokens: Vec<Token>, pos: usize }
impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self { Parser { tokens, pos: 0 } }
    pub fn parse_program(&mut self) -> Vec<Statement> {
        let mut stmts = Vec::new();
        while !self.is_at_end() { stmts.push(self.parse_statement()); }
        stmts
    }
    fn parse_statement(&mut self) -> Statement {
        if self.match_token(Token::Fn) { return self.function_define(); }
        if self.match_token(Token::Let) { return self.let_statement(); }
        if self.match_token(Token::Root) { return self.root_statement(); }
        if self.match_token(Token::Loop) { return self.loop_statement(); }
        if self.match_token(Token::While) { return self.while_statement(); }
        if self.match_token(Token::If) { return self.if_statement(); }
        if self.match_token(Token::Break) { self.consume(Token::SemiColon); return Statement::Break; }
        if self.match_token(Token::Return) { self.consume(Token::SemiColon); return Statement::Return; }
        if self.match_token(Token::Outb) { return self.outb_stmt(); }
        if self.match_token(Token::Poke) { return self.poke_stmt(); }
        if self.match_token(Token::Asm) { return self.asm_stmt(); }
        let token = self.advance();
        if let Token::Identifier(id) = token {
            if self.match_token(Token::LParen) { return self.call_stmt(id); }
            if self.match_token(Token::Equal) { return self.assign_stmt(id); }
        }
        panic!("Error at {}", self.pos);
    }
    fn outb_stmt(&mut self) -> Statement {
        self.consume(Token::LParen);
        let port = self.parse_expression(); self.consume(Token::Comma);
        let val = self.parse_expression(); self.consume(Token::RParen);
        self.consume(Token::SemiColon);
        Statement::Outb(port, val)
    }
    fn poke_stmt(&mut self) -> Statement {
        self.consume(Token::LParen);
        let addr = self.parse_expression(); self.consume(Token::Comma);
        let val = self.parse_expression(); self.consume(Token::RParen);
        self.consume(Token::SemiColon);
        Statement::Poke(addr, val)
    }
    fn function_define(&mut self) -> Statement {
        let name = if let Token::Identifier(s) = self.advance() { s } else { panic!() };
        self.consume(Token::LParen);
        let mut params = Vec::new();
        if !self.check(Token::RParen) {
            loop {
                if let Token::Identifier(p) = self.advance() { params.push(p); }
                if !self.match_token(Token::Comma) { break; }
            }
        }
        self.consume(Token::RParen); self.consume(Token::LBrace);
        let mut body = Vec::new();
        while !self.check(Token::RBrace) && !self.is_at_end() { body.push(self.parse_statement()); }
        self.consume(Token::RBrace);
        Statement::FunctionDefine(name, params, body)
    }
    fn parse_expression(&mut self) -> Expression {
        let mut expr = self.primary();
        while self.match_any(&["+", "-", "*", "/", "==", ">", "<"]) {
            let op = self.previous_op();
            let right = self.primary();
            expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
        }
        expr
    }
    fn primary(&mut self) -> Expression {
        match self.peek() {
            Token::Number(n) => { self.advance(); Expression::Number(n) }
            Token::Identifier(s) if s == "in" => { self.advance(); Expression::WaitKey }
            Token::Identifier(s) => { self.advance(); Expression::Variable(s) }
            Token::Peek => { self.advance(); self.consume(Token::LParen); let addr = self.parse_expression(); self.consume(Token::RParen); Expression::Peek(Box::new(addr)) }
            _ => panic!()
        }
    }
    fn let_statement(&mut self) -> Statement {
        let n = if let Token::Identifier(s) = self.advance() { s } else { panic!() };
        self.consume(Token::Equal);
        let v = self.parse_expression(); self.consume(Token::SemiColon);
        Statement::Let(n, v)
    }
    fn root_statement(&mut self) -> Statement {
        let n = if let Token::Identifier(s) = self.advance() { s } else { panic!() };
        self.consume(Token::Equal);
        let v = self.parse_expression(); self.consume(Token::SemiColon);
        Statement::Root(n, v)
    }
    fn loop_statement(&mut self) -> Statement {
        self.consume(Token::LBrace);
        let mut b = Vec::new();
        while !self.check(Token::RBrace) { b.push(self.parse_statement()); }
        self.consume(Token::RBrace);
        Statement::Loop(b)
    }
    fn while_statement(&mut self) -> Statement {
        let cond = self.parse_expression(); self.consume(Token::LBrace);
        let mut body = Vec::new();
        while !self.check(Token::RBrace) { body.push(self.parse_statement()); }
        self.consume(Token::RBrace);
        Statement::While(cond, body)
    }
    fn if_statement(&mut self) -> Statement {
        let c = self.parse_expression(); self.consume(Token::LBrace);
        let mut then_body = Vec::new();
        while !self.check(Token::RBrace) { then_body.push(self.parse_statement()); }
        self.consume(Token::RBrace);
        let else_body = if self.match_token(Token::Else) {
            self.consume(Token::LBrace);
            let mut e = Vec::new();
            while !self.check(Token::RBrace) { e.push(self.parse_statement()); }
            self.consume(Token::RBrace);
            Some(e)
        } else {
            None
        };
        Statement::If(c, then_body, else_body)
    }
    fn call_stmt(&mut self, name: String) -> Statement {
        let mut args = Vec::new();
        if !self.check(Token::RParen) {
            loop { args.push(self.parse_expression()); if !self.match_token(Token::Comma) { break; } }
        }
        self.consume(Token::RParen); self.consume(Token::SemiColon);
        Statement::Call(name, args)
    }
    fn assign_stmt(&mut self, name: String) -> Statement {
        let v = self.parse_expression(); self.consume(Token::SemiColon);
        Statement::Assignment(name, v)
    }
    fn asm_stmt(&mut self) -> Statement {
        self.consume(Token::LParen);
        let c = if let Token::StringLiteral(s) = self.advance() { s } else { panic!() };
        self.consume(Token::RParen); self.consume(Token::SemiColon);
        Statement::Asm(c)
    }
    fn match_token(&mut self, t: Token) -> bool { if self.check(t) { self.advance(); true } else { false } }
    fn check(&self, t: Token) -> bool { self.peek() == t }
    fn peek(&self) -> Token { self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF) }
    fn advance(&mut self) -> Token { if !self.is_at_end() { self.pos += 1; } self.tokens[self.pos-1].clone() }
    fn is_at_end(&self) -> bool { self.peek() == Token::EOF }
    fn match_any(&mut self, ops: &[&str]) -> bool {
        let c = self.peek();
        for op in ops {
            match *op {
                "+" if c == Token::Plus => { self.advance(); return true; }
                "-" if c == Token::Minus => { self.advance(); return true; }
                "*" if c == Token::Star => { self.advance(); return true; }
                "/" if c == Token::Slash => { self.advance(); return true; }
                "==" if c == Token::EqEq => { self.advance(); return true; }
                ">" if c == Token::Greater => { self.advance(); return true; }
                "<" if c == Token::Less => { self.advance(); return true; }
                _ => {}
            }
        }
        false
    }
    fn previous_op(&self) -> String {
        match self.tokens.get(self.pos-1) {
            Some(Token::Plus) => "+".into(), Some(Token::Minus) => "-".into(),
            Some(Token::Star) => "*".into(), Some(Token::Slash) => "/".into(),
            Some(Token::EqEq) => "==".into(), Some(Token::Greater) => ">".into(),
            Some(Token::Less) => "<".into(), _ => "".into()
        }
    }
    fn consume(&mut self, t: Token) { if !self.match_token(t) { panic!(); } }
}