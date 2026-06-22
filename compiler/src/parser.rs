    use crate::lexer::Token;
    use crate::ast::{Statement, Expression, TypeKind};

    pub struct Parser { 
        tokens: Vec<Token>, 
        pos: usize 
    }

    impl Parser {
        pub fn new(tokens: Vec<Token>) -> Self { 
            eprintln!("[INFO] Parser initialized with {} tokens", tokens.len());
            Parser { tokens, pos: 0 } 
        }

    fn peek_token(&self) -> Option<Token> {
        if self.pos < self.tokens.len() {
            Some(self.tokens[self.pos].clone())
        } else { 
            None
        }
    }

    fn token_to_string(&self, tok: Token) -> String {
        match tok {
            Token::EqEq => "==".to_string(),
            Token::NotEq => "!=".to_string(),
            Token::Less => "<".to_string(),
            Token::Greater => ">".to_string(),
            Token::LessEq => "<=".to_string(),
            Token::GreaterEq => ">=".to_string(),
            Token::Plus => "+".to_string(),
            Token::Minus => "-".to_string(),
            Token::Star => "*".to_string(),
            Token::Slash => "/".to_string(),
            Token::Ampersand => "&".to_string(),
            Token::ShiftLeft => "<<".to_string(),
            Token::ShiftRight => ">>".to_string(),
            Token::Pipe => "|".to_string(),
            Token::Caret => "^".to_string(),
            _ => "".to_string(),
        }
    }




   fn parse_type(&mut self) -> TypeKind {
    match self.peek() {
        Token::U8  => { self.advance(); TypeKind::U8  }
        Token::U16 => { self.advance(); TypeKind::U16 }
        Token::U32 => { self.advance(); TypeKind::U32 }
        Token::U64 => { self.advance(); TypeKind::U64 }
        Token::I8  => { self.advance(); TypeKind::I8  }
        Token::I16 => { self.advance(); TypeKind::I16 }
        Token::I32 => { self.advance(); TypeKind::I32 }
        Token::I64 => { self.advance(); TypeKind::I64 }
        Token::Bool => { self.advance(); TypeKind::Bool }
        _ => {
            eprintln!("\x1b[33m[TYPE WARNING] No type specified, defaulting to u32\x1b[0m");
            TypeKind::Unknown
        }
    }
}

        
        pub fn parse_program(&mut self) -> Vec<Statement> {
    let mut stmts = Vec::new();
    while !self.is_at_end() {
        stmts.push(self.parse_statement());
    }
    stmts
}
        
    fn return_statement(&mut self) -> Statement {
            if self.match_token(Token::SemiColon) {
                return Statement::Return(None);
            }
            let expr = self.parse_expression();
            self.consume(Token::SemiColon);
            Statement::Return(Some(expr))
        }

    fn parse_statement(&mut self) -> Statement {
        if self.match_token(Token::Fn) { return self.function_define(); }
        if self.match_token(Token::Let) { return self.let_or_array_or_string(); }
        if self.match_token(Token::Root) { return self.root_statement(); }
        if self.match_token(Token::Loop) { return self.loop_statement(); }
        if self.match_token(Token::While) { return self.while_statement(); }
        if self.match_token(Token::If) { return self.if_statement(); }
        if self.match_token(Token::Break) { self.consume(Token::SemiColon); return Statement::Break; }
        if self.match_token(Token::Return) { return self.return_statement(); }
        if self.match_token(Token::Outb) { return self.outb_stmt(); }
        if self.match_token(Token::Poke) { return self.poke_stmt(); }
        if self.match_token(Token::Asm) { return self.asm_stmt(); }
        if self.match_token(Token::Call) { return self.callptr_stmt(); }
        if self.match_token(Token::Struct) { return self.parse_struct(); }
    
        if self.match_token(Token::Int)        { return self.int_handler_stmt(); }
        if self.match_token(Token::IntEnable)  { return self.int_enable_stmt(); }
        if self.match_token(Token::IntDisable) { self.consume(Token::SemiColon); return Statement::IntDisable; }

        if self.match_token(Token::SaveCtx) {
            if let Token::Identifier(name) = self.advance() {
                self.consume(Token::SemiColon);
                return Statement::SaveContext(name);
            }
        }
        if self.match_token(Token::RestoreCtx) {
            if let Token::Identifier(name) = self.advance() {
                self.consume(Token::SemiColon);
                return Statement::RestoreContext(name);
            }
        }
        
if self.match_token(Token::Bnw) {
    if let Token::StringLiteral(s) = self.advance() {
        self.consume(Token::SemiColon);
        return Statement::Bnw(s);
    } else {
        eprintln!("[BNW ERROR] Expected string after 'bnw'");
        std::process::exit(1);
    }
}
        let token = self.advance();
        if let Token::Identifier(id) = token {
            
            if self.match_token(Token::LParen) { return self.call_stmt(id); }

            if self.match_token(Token::Dot) {
    let field = if let Token::Identifier(f) = self.advance() { f } else {
        eprintln!("[PARSER ERROR] Expected field name after '.'");
        std::process::exit(1);
    };
    self.consume(Token::Equal);
    let val = self.parse_expression();
    self.consume(Token::SemiColon);
    return Statement::Assignment(format!("{}.{}", id, field), val);
}
         
            if self.match_token(Token::LBracket) {
                let idx = self.parse_expression();
                self.consume(Token::RBracket);
                self.consume(Token::Equal);
                let val = self.parse_expression();
                self.consume(Token::SemiColon);
                return Statement::ArrayAssign(id, idx, val); 
            }

    
            if self.match_token(Token::Equal) { return self.assign_stmt(id); }
            
            eprintln!("[PARSER ERROR] Invalid statement starting with identifier '{}'", id);
            std::process::exit(1);
        } else {
            eprintln!("[PARSER ERROR] Unexpected token: {:?}", token);
            std::process::exit(1);
        }
    }
        
        fn let_or_array_or_string(&mut self) -> Statement {
    let n = if let Token::Identifier(s) = self.advance() {
        s
    } else {
        std::process::exit(1);
        
    };



    let kind = if self.match_token(Token::At) {
        if let Token::Identifier(struct_name) = self.peek() {

            self.advance();
            self.consume(Token::SemiColon);
            return Statement::StructInstance(n, struct_name);
        }
        let k = self.parse_type();
        if k == TypeKind::Unknown {
            eprintln!("[PARSER ERROR] Expected type after '@'");
            std::process::exit(1);
        }
        k
    } else {
        eprintln!("[TYPE WARNING] '{}' has no type annotation, defaulting to u32", n);
        TypeKind::Unknown
    };

    self.consume(Token::Equal);


    if self.match_token(Token::LBracket) {
        let mut vals = Vec::new();
        while !self.check(Token::RBracket) {
            if let Token::Number(num) = self.advance() {
                vals.push(num);
            } else {
                eprintln!("[PARSER ERROR] Expected number in array literal");
                std::process::exit(1);
            }
            if !self.match_token(Token::Comma) { break; }
        }
        self.consume(Token::RBracket);
        self.consume(Token::SemiColon);
        return Statement::ArrayDefine(n, vals, kind);
    }


    if let Token::StringLiteral(s) = self.peek() {
        self.advance();
        self.consume(Token::SemiColon);
        return Statement::StringDefine(n, s);
    }

    let v = self.parse_expression();


    if let Expression::Number(num, _) = &v {
        if kind != TypeKind::Unknown && *num > kind.max_value() {
            eprintln!("[TYPE ERROR] Value {} exceeds max value for type {} (max: {})",
                num, kind.name(), kind.max_value());
            std::process::exit(1);
        }
    }

    self.consume(Token::SemiColon);
    Statement::Let(n, v, kind)
}
        
        fn outb_stmt(&mut self) -> Statement {
            self.consume(Token::LParen);
            let port = self.parse_expression(); 
            self.consume(Token::Comma);
            let val = self.parse_expression(); 
            self.consume(Token::RParen);
            self.consume(Token::SemiColon);
            Statement::Outb(port, val)
        }
        
        fn poke_stmt(&mut self) -> Statement {
            self.consume(Token::LParen);
            let addr = self.parse_expression(); 
            self.consume(Token::Comma);
            let val = self.parse_expression(); 
            self.consume(Token::RParen);
            self.consume(Token::SemiColon);
            Statement::Poke(addr, val)
        }
        
        fn function_define(&mut self) -> Statement {
    let name = if let Token::Identifier(s) = self.advance() {
        s
    } else {
        eprintln!("[PARSER ERROR] Expected function name after 'fn'");
        std::process::exit(1);
    };

    self.consume(Token::LParen);
    let mut params = Vec::new();

    if !self.check(Token::RParen) {
        loop {
            let pname = if let Token::Identifier(p) = self.advance() {
                p
            } else {
                eprintln!("[PARSER ERROR] Expected parameter name");
                std::process::exit(1);
            };

            let pkind = if self.match_token(Token::At) {
                self.parse_type()
            } else {
                TypeKind::Unknown
            };
            params.push((pname, pkind));
            if !self.match_token(Token::Comma) { break; }
        }
    }

    self.consume(Token::RParen);


    let return_type = if self.match_token(Token::At) {
        self.parse_type()
    } else {
        TypeKind::Unknown
    };

    self.consume(Token::LBrace);
    let mut body = Vec::new();
    while !self.check(Token::RBrace) && !self.is_at_end() {
        body.push(self.parse_statement());
    }
    self.consume(Token::RBrace);

    Statement::FunctionDefine(name, params, body, return_type)
}
   
  fn parse_expression(&mut self) -> Expression {
    let mut expr = self.parse_term();

    while let Some(token) = self.peek_token() {
        match token {
            Token::EqEq | Token::NotEq | Token::Greater | Token::Less | 
            Token::GreaterEq | Token::LessEq => {
                let current_token = self.advance(); 
                let op = self.token_to_string(current_token); 
                
                let right = self.parse_term();
                expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
            }
            _ => break,
        }
    }
    expr
}
 
    fn parse_term(&mut self) -> Expression {
    let mut expr = self.parse_factor(); 

    while let Some(token) = self.peek_token() {
        match token {
  
            Token::Plus | Token::Minus | Token::Pipe | Token::Caret | Token::Ampersand => {
                let current_token = self.advance();
                let op = self.token_to_string(current_token);
                let right = self.parse_factor();
                expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
            }
            _ => break,
        }
    }
    expr
}


    fn parse_factor(&mut self) -> Expression {
    let mut expr = self.primary(); 

    while let Some(token) = self.peek_token() {
        match token {
       
            Token::Star | Token::Slash | Token::ShiftLeft | Token::ShiftRight => {
                let current_token = self.advance();
                let op = self.token_to_string(current_token);
                let right = self.primary();
                expr = Expression::BinaryOp(Box::new(expr), op, Box::new(right));
            }
            _ => break,
        }
    }
    expr
}


    fn primary(&mut self) -> Expression {
        match self.peek() {
   
            Token::LParen => {
                self.advance(); 
                let expr = self.parse_expression();
                self.consume(Token::RParen);
                expr
            }
            
         Token::Number(n) => { 
    self.advance(); 
    Expression::Number(n, TypeKind::Unknown) 
               }
            

        Token::Identifier(s) => {  
        self.advance();  
        
    
        if self.match_token(Token::LParen) {
            let mut args = Vec::new();
            if !self.check(Token::RParen) {
                loop {
                    args.push(self.parse_expression());
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::RParen);
           
            return Expression::Call(s, args); 
        }

   
        if self.match_token(Token::LBracket) {
            let idx = self.parse_expression();  
            self.consume(Token::RBracket);
            return Expression::ArrayAccess(s, Box::new(idx));
        }
        if self.match_token(Token::Dot) {
    let field = if let Token::Identifier(f) = self.advance() { f } else {
        eprintln!("[PARSER ERROR] Expected field name after '.'");
        std::process::exit(1);
    };
    return Expression::FieldAccess(s, field);
}

        Expression::Variable(s)  
    }


            Token::Peek => { 
                self.advance(); 
                self.consume(Token::LParen); 
                let addr = self.parse_expression(); 
                self.consume(Token::RParen); 
                Expression::Peek(Box::new(addr)) 
            } 
            Token::Inb => {
        self.advance();
        self.consume(Token::LParen);
        let port = self.parse_expression();
        self.consume(Token::RParen);
        Expression::Inb(Box::new(port))
    }

Token::Ampersand => {
                self.advance();
                if let Token::Identifier(name) = self.advance() {
                    Expression::AddressOf(name)
                } else {
                    eprintln!("[PARSER ERROR] Expected function name after '&'");
                    std::process::exit(1);
                }
            }

            _ => {
                eprintln!("[PARSER ERROR] Unexpected token in expression: {:?}", self.peek());
                eprintln!("[HINT] Expected: number, variable, (, or peek(...)");
                std::process::exit(1);
            }
        }
    }
        
      fn root_statement(&mut self) -> Statement {
    let n = if let Token::Identifier(s) = self.advance() {
        s
    } else {
        eprintln!("[PARSER ERROR] Expected identifier after 'root'");
        std::process::exit(1);
    };


    let kind = if self.match_token(Token::At) {
        self.parse_type()
    } else {
        TypeKind::Unknown
    };

    self.consume(Token::Equal);
    let v = self.parse_expression();
    self.consume(Token::SemiColon);
    Statement::Root(n, v, kind)
}

fn loop_statement(&mut self) -> Statement {
    self.consume(Token::LBrace);
    let mut b = Vec::new();
    while !self.check(Token::RBrace) {
        b.push(self.parse_statement());
    }
    self.consume(Token::RBrace);
    Statement::Loop(b)
}
        
        fn while_statement(&mut self) -> Statement {
            self.consume(Token::LParen); // استهلاك (
            let cond = self.parse_expression(); 
            self.consume(Token::RParen); // استهلاك )
            
            self.consume(Token::LBrace);
            let mut body = Vec::new();
            while !self.check(Token::RBrace) && !self.is_at_end() { 
                body.push(self.parse_statement()); 
            }
            self.consume(Token::RBrace);
            Statement::While(cond, body)
        }
        
    fn if_statement(&mut self) -> Statement {
            self.consume(Token::LParen);
            let c = self.parse_expression(); 
            self.consume(Token::RParen);
            
            self.consume(Token::LBrace);
            let mut then_body = Vec::new();
            while !self.check(Token::RBrace) && !self.is_at_end() { 
                then_body.push(self.parse_statement()); 
            }
            self.consume(Token::RBrace);
            
            let else_body = if self.match_token(Token::Else) {
                self.consume(Token::LBrace);
                let mut e = Vec::new();
                while !self.check(Token::RBrace) && !self.is_at_end() { 
                    e.push(self.parse_statement()); 
                }
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
                loop { 
                    args.push(self.parse_expression()); 
                    if !self.match_token(Token::Comma) { break; } 
                }
            }
            self.consume(Token::RParen); 
            self.consume(Token::SemiColon);
            Statement::Call(name, args)
        }
        
        fn assign_stmt(&mut self, name: String) -> Statement {
            let v = self.parse_expression(); 
            self.consume(Token::SemiColon);
            Statement::Assignment(name, v)
        }
        
        fn asm_stmt(&mut self) -> Statement {
            self.consume(Token::LParen);
            
          
            let token = self.advance();
            let c = if let Token::StringLiteral(s) = token {
                s
            } else {
                eprintln!("[PARSER ERROR] Expected string literal for asm, got {:?}", token);
               std::process::exit(1);
            };
            
            self.consume(Token::RParen); 
            self.consume(Token::SemiColon);
            Statement::Asm(c)
        }
 
        fn callptr_stmt(&mut self) -> Statement {
    self.consume(Token::LParen);
    let expr = self.parse_expression();
    self.consume(Token::RParen);
    self.consume(Token::SemiColon);
    Statement::CallPtr(expr)
}
        
        fn match_token(&mut self, t: Token) -> bool { 
            if self.check(t) { 
                self.advance(); 
                true 
            } else { 
                false 
            } 
        }
        
        fn check(&self, t: Token) -> bool { 
            self.peek() == t 
        }
        
        fn peek(&self) -> Token { 
            self.tokens.get(self.pos).cloned().unwrap_or(Token::EOF) 
        }
        
        fn advance(&mut self) -> Token { 
            if !self.is_at_end() { 
                self.pos += 1; 
            } 
            self.tokens[self.pos-1].clone() 
        }
        
        fn is_at_end(&self) -> bool { 
            self.peek() == Token::EOF 
        }
        
        #[allow(dead_code)]
        fn match_any(&mut self, ops: &[&str]) -> bool {
            let c = self.peek();
            for op in ops {
                match *op {
                    "^" if c == Token::Caret => { self.advance(); return true; }
                    "+" if c == Token::Plus => { self.advance(); return true; }
                    "-" if c == Token::Minus => { self.advance(); return true; }
                    "*" if c == Token::Star => { self.advance(); return true; }
                    "/" if c == Token::Slash => { self.advance(); return true; }
                    "!=" if c == Token::NotEq => { self.advance(); return true; }
                    "<<" if c == Token::ShiftLeft => { self.advance(); return true; }
                    ">>" if c == Token::ShiftRight => { self.advance(); return true; }
                    ">=" if c == Token::GreaterEq => { self.advance(); return true; }
                    "<=" if c == Token::LessEq => { self.advance(); return true; }
                    "==" if c == Token::EqEq => { self.advance(); return true; }
                    ">" if c == Token::Greater => { self.advance(); return true; }
                    "<" if c == Token::Less => { self.advance(); return true; }
                    "&" if c == Token::Ampersand => { self.advance(); return true; }
                    "|" if c == Token::Pipe => { self.advance(); return true; }
                    _ => {}
                }
            }
            false
        }
        
        #[allow(dead_code)]
        fn previous_op(&self) -> String {
            match self.tokens.get(self.pos-1) {
                Some(Token::Caret) => "^".into(),
                Some(Token::Plus) => "+".into(), 
                Some(Token::Minus) => "-".into(),
                Some(Token::ShiftLeft) => "<<".into(),
                Some(Token::ShiftRight) => ">>".into(),
                Some(Token::Star) => "*".into(), 
                Some(Token::Slash) => "/".into(),
                Some(Token::EqEq) => "==".into(), 
                Some(Token::NotEq) => "!=".into(),
                Some(Token::GreaterEq) => ">=".into(),
                Some(Token::LessEq) => "<=".into(),
                Some(Token::Greater) => ">".into(),
                Some(Token::Less) => "<".into(), 
                Some(Token::Ampersand) => "&".into(),
                Some(Token::Pipe) => "|".into(), 
                _ => "".into()
            }
        }
fn parse_struct(&mut self) -> Statement {
    let name = if let Token::Identifier(s) = self.advance() { s } else {
        eprintln!("[PARSER ERROR] Expected struct name");
        std::process::exit(1);
    };
    self.consume(Token::LBrace);
    let mut fields = Vec::new();
    while !self.check(Token::RBrace) && !self.is_at_end() {
        let field_name = if let Token::Identifier(s) = self.advance() { s } else {
            eprintln!("[PARSER ERROR] Expected field name");
            std::process::exit(1);
        };
        self.consume(Token::At);
        let kind = self.parse_type();
        fields.push((field_name, kind));
        if self.check(Token::Comma) { self.advance(); }
    }
    self.consume(Token::RBrace);
    Statement::StructDefine(name, fields)
}

        fn consume(&mut self, t: Token) { 
            if !self.match_token(t.clone()) { 
                eprintln!("[PARSER ERROR] Expected {:?}, got {:?} at position {}", 
                    t, self.peek(), self.pos);
                std::process::exit(1);
            } 
        }

fn int_handler_stmt(&mut self) -> Statement {
    let name = if let Token::Identifier(s) = self.advance() { s } else {
        eprintln!("[PARSER ERROR] Expected handler name after 'int'");
        std::process::exit(1);
    };
    self.consume(Token::LBrace);
    let mut body = Vec::new();
    while !self.check(Token::RBrace) && !self.is_at_end() {
        body.push(self.parse_statement());
    }
    self.consume(Token::RBrace);
    Statement::IntHandler(name, body)
}

fn int_enable_stmt(&mut self) -> Statement {
    self.consume(Token::LParen);
    let vector_id = self.parse_expression();
    self.consume(Token::Comma);
    let handler_name = if let Token::Identifier(s) = self.advance() { s } else {
        eprintln!("[PARSER ERROR] Expected handler name in int_enable");
        std::process::exit(1);
    };
    self.consume(Token::RParen);
    self.consume(Token::SemiColon);
    Statement::IntEnable(vector_id, handler_name)
}
    }

