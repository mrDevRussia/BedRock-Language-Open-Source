    use crate::lexer::Token;
    use crate::ast::{Statement, Expression};

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
        
        pub fn parse_program(&mut self) -> Vec<Statement> {
            let mut stmts = Vec::new();
            while !self.is_at_end() { 
                match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                    self.parse_statement()
                })) {
                    Ok(stmt) => stmts.push(stmt),
                    Err(_) => {
                        eprintln!("[PARSER ERROR] Failed at token position: {}", self.pos);
                        if self.pos < self.tokens.len() {
                            eprintln!("[PARSER ERROR] Current token: {:?}", self.tokens[self.pos]);
                        }
                        panic!("Parse error - see details above");
                    }
                }
            }
            stmts
        }
        
    fn return_statement(&mut self) -> Statement {
            if self.match_token(Token::SemiColon) {
                return Statement::Return(None); // أضف (None) هنا
            }
            let expr = self.parse_expression();
            self.consume(Token::SemiColon);
            Statement::Return(Some(expr)) // أضف Some(expr) هنا
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
        
        let token = self.advance();
        if let Token::Identifier(id) = token {
            // 1. استدعاء دالة: func();
            if self.match_token(Token::LParen) { return self.call_stmt(id); }
            
            // 2. ✅ التعديل الجديد: تخصيص قيمة لعنصر في مصفوفة: arr[0] = 5;
            if self.match_token(Token::LBracket) {
                let idx = self.parse_expression();
                self.consume(Token::RBracket);
                self.consume(Token::Equal);
                let val = self.parse_expression();
                self.consume(Token::SemiColon);
                return Statement::ArrayAssign(id, idx, val); 
            }

            // 3. مساواة عادية: x = 10;
            if self.match_token(Token::Equal) { return self.assign_stmt(id); }
            
            eprintln!("[PARSER ERROR] Invalid statement starting with identifier '{}'", id);
            panic!("Invalid syntax");
        } else {
            eprintln!("[PARSER ERROR] Unexpected token: {:?}", token);
            panic!("Parse error");
        }
    }
        
        fn let_or_array_or_string(&mut self) -> Statement {
            let n = if let Token::Identifier(s) = self.advance() { 
                s 
            } else { 
                eprintln!("[PARSER ERROR] Expected identifier after 'let'");
                eprintln!("[HINT] Syntax: let name = value;");
                panic!("Invalid let statement");
            };
            
            self.consume(Token::Equal);
            
            if self.match_token(Token::LBracket) {
                let mut vals = Vec::new();
                while !self.check(Token::RBracket) {
                    if let Token::Number(num) = self.advance() { 
                        vals.push(num); 
                    } else {
                        eprintln!("[PARSER ERROR] Expected number in array literal");
                        panic!("Invalid array element");
                    }
                    if !self.match_token(Token::Comma) { break; }
                }
                self.consume(Token::RBracket); 
                self.consume(Token::SemiColon);
                return Statement::ArrayDefine(n, vals);
            }
            
            if let Token::StringLiteral(s) = self.peek() {
                self.advance(); 
                self.consume(Token::SemiColon);
                return Statement::StringDefine(n, s);
            }
            
            let v = self.parse_expression(); 
            self.consume(Token::SemiColon);
            Statement::Let(n, v)
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
                panic!("Invalid function definition");
            };
            
            self.consume(Token::LParen);
            let mut params = Vec::new();
            
            if !self.check(Token::RParen) {
                loop {
                    if let Token::Identifier(p) = self.advance() { 
                        params.push(p); 
                    } else {
                        eprintln!("[PARSER ERROR] Expected parameter name");
                        panic!("Invalid parameter");
                    }
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            
            self.consume(Token::RParen); 
            self.consume(Token::LBrace);
            
            let mut body = Vec::new();
            while !self.check(Token::RBrace) && !self.is_at_end() { 
                body.push(self.parse_statement()); 
            }
            
            self.consume(Token::RBrace);
            Statement::FunctionDefine(name, params, body)
        }
        
    // 1. التعبير يبدأ بالبحث عن العمليات المنطقية (أصغر، أكبر، يساوي)
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
    // 2. معالجة الجمع والطرح (أولوية متوسطة)
    fn parse_term(&mut self) -> Expression {
    let mut expr = self.parse_factor(); 

    while let Some(token) = self.peek_token() {
        match token {
            // تأكد من وجود Pipe (|) و Caret (^) و Ampersand (&) هنا للعمليات المنطقية
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

    // 3. معالجة الضرب والقسمة (أولوية عالية)
    fn parse_factor(&mut self) -> Expression {
    let mut expr = self.primary(); 

    while let Some(token) = self.peek_token() {
        match token {
            // أضف ShiftLeft و ShiftRight هنا
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
            // ✅ الإضافة الجديدة: التعامل مع الأقواس
            Token::LParen => {
                self.advance(); // تخطي القوس (
                let expr = self.parse_expression(); // قراءة ما بداخل القوس كـ Expression كامل
                self.consume(Token::RParen); // التأكد من وجود قوس إغلاق )
                expr
            }
            
            Token::Number(n) => { 
                self.advance(); 
                Expression::Number(n) 
            }


        Token::Identifier(s) => {  
        self.advance();  
        
        // التعديل السحري: هل هذا استدعاء دالة؟
        if self.match_token(Token::LParen) {
            let mut args = Vec::new();
            if !self.check(Token::RParen) {
                loop {
                    args.push(self.parse_expression());
                    if !self.match_token(Token::Comma) { break; }
                }
            }
            self.consume(Token::RParen);
            // نحن نحتاج لإرجاع استدعاء دالة كتعبير
            return Expression::Call(s, args); 
        }

        // هل هو وصول لمصفوفة؟
        if self.match_token(Token::LBracket) {
            let idx = self.parse_expression();  
            self.consume(Token::RBracket);
            return Expression::ArrayAccess(s, Box::new(idx));
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

            _ => {
                eprintln!("[PARSER ERROR] Unexpected token in expression: {:?}", self.peek());
                eprintln!("[HINT] Expected: number, variable, (, or peek(...)");
                panic!("Invalid expression");
            }
        }
    }
        
        fn root_statement(&mut self) -> Statement {
            let n = if let Token::Identifier(s) = self.advance() { 
                s 
            } else { 
                eprintln!("[PARSER ERROR] Expected identifier after 'root'");
                panic!("Invalid root declaration");
            };
            
            self.consume(Token::Equal);
            let v = self.parse_expression(); 
            self.consume(Token::SemiColon);
            Statement::Root(n, v)
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
            self.consume(Token::LParen); // يجب استهلاك القوس ( أولاً
            let c = self.parse_expression(); 
            self.consume(Token::RParen); // يجب استهلاك القوس ) بعد الشرط
            
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
            
            // إصلاح: تحقق من StringLiteral بشكل صحيح
            let token = self.advance();
            let c = if let Token::StringLiteral(s) = token {
                s
            } else {
                eprintln!("[PARSER ERROR] Expected string literal for asm, got {:?}", token);
                panic!("Invalid asm statement");
            };
            
            self.consume(Token::RParen); 
            self.consume(Token::SemiColon);
            Statement::Asm(c)
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

        fn consume(&mut self, t: Token) { 
            if !self.match_token(t.clone()) { 
                eprintln!("[PARSER ERROR] Expected {:?}, got {:?} at position {}", 
                    t, self.peek(), self.pos);
                panic!("Syntax error");
            } 
        }
    }

