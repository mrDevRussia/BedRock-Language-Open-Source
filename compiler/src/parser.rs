use crate::lexer::{Lexer, Token};
use crate::ast::*;

pub struct Parser {
    lexer: Lexer,
    current_token: Token,
}

impl Parser {
    pub fn new(mut lexer: Lexer) -> Self {
        let current_token = lexer.next_token();
        Parser { lexer, current_token }
    }

    fn advance(&mut self) {
        self.current_token = self.lexer.next_token();
    }

    fn expect(&mut self, expected: Token) -> Result<(), String> {
        if std::mem::discriminant(&self.current_token) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, self.current_token))
        }
    }

    fn parse_type(&mut self) -> Result<Type, String> {
        match &self.current_token {
            Token::Star => {
                self.advance();
                let inner = self.parse_type()?;
                Ok(Type::Pointer(Box::new(inner)))
            }
            Token::Identifier(name) => {
                let ty = match name.as_str() {
                    "u8" => Type::U8,
                    "u16" => Type::U16,
                    "u32" => Type::U32,
                    "u64" => Type::U64,
                    "i8" => Type::I8,
                    "i16" => Type::I16,
                    "i32" => Type::I32,
                    "i64" => Type::I64,
                    "f32" => Type::F32,
                    "f64" => Type::F64,
                    "bool" => Type::Bool,
                    "void" => Type::Void,
                    _ => return Err(format!("Unknown type: {}", name)),
                };
                self.advance();
                Ok(ty)
            }
            _ => Err(format!("Expected type, found {:?}", self.current_token)),
        }
    }

    pub fn parse_program(&mut self) -> Result<Program, String> {
        let mut items = Vec::new();
        while self.current_token != Token::EOF {
            items.push(self.parse_top_level_item()?);
        }
        Ok(Program { items })
    }

    fn parse_attributes(&mut self) -> Result<Vec<Attribute>, String> {
        let mut attributes = Vec::new();
        while self.current_token == Token::Hash {
            self.advance(); // consume #
            self.expect(Token::LBracket)?;
            
            if let Token::Identifier(name) = &self.current_token {
                let attr_name = name.clone();
                self.advance();
                match attr_name.as_str() {
                    "address" => {
                        self.expect(Token::LParen)?;
                        if let Token::Integer(addr) = self.current_token {
                            attributes.push(Attribute::Address(addr));
                            self.advance();
                        } else {
                            return Err("Expected integer address".to_string());
                        }
                        self.expect(Token::RParen)?;
                    }
                    "interrupt" => {
                        attributes.push(Attribute::Interrupt);
                    }
                    "align" => {
                        self.expect(Token::LParen)?;
                         if let Token::Integer(val) = self.current_token {
                            attributes.push(Attribute::Align(val));
                            self.advance();
                        } else {
                            return Err("Expected integer alignment".to_string());
                        }
                        self.expect(Token::RParen)?;
                    }
                    _ => return Err(format!("Unknown attribute: {}", attr_name)),
                }
            } else {
                return Err("Expected attribute name".to_string());
            }
            self.expect(Token::RBracket)?;
        }
        Ok(attributes)
    }

    fn parse_top_level_item(&mut self) -> Result<TopLevelItem, String> {
        let attributes = self.parse_attributes()?;

        let is_volatile = if self.current_token == Token::Volatile {
            self.advance();
            true
        } else {
            false
        };

        if self.current_token == Token::Let {
            // Global Variable
            self.advance(); // consume let
            let name = if let Token::Identifier(n) = &self.current_token {
                n.clone()
            } else {
                return Err("Expected variable name".to_string());
            };
            self.advance();
            
            self.expect(Token::Colon)?;
            let ty = self.parse_type()?;
            self.expect(Token::SemiColon)?;
            
            Ok(TopLevelItem::GlobalVariable(GlobalVariable {
                name,
                ty,
                is_volatile,
                attributes,
            }))
        } else if self.current_token == Token::Fn {
            // Function
            if is_volatile {
                return Err("Functions cannot be volatile".to_string());
            }
            self.advance(); // consume fn
            let name = if let Token::Identifier(n) = &self.current_token {
                n.clone()
            } else {
                return Err("Expected function name".to_string());
            };
            self.advance();

            self.expect(Token::LParen)?;
            // TODO: Parse args if needed, but example has none or I'll implement basic skip/check
            // For now assume empty args for kernel_main or implement arg parsing later
            // Example: fn keyboard_handler() -> void
            self.expect(Token::RParen)?;

            self.expect(Token::Arrow)?;
            let return_type = self.parse_type()?;
            
            self.expect(Token::LBrace)?;
            let body = self.parse_block()?;
            // RBrace consumed in parse_block or checked? 
            // Actually parse_block usually consumes RBrace
            
            Ok(TopLevelItem::Function(Function {
                name,
                return_type,
                attributes,
                body,
            }))
        } else {
            Err(format!("Unexpected token at top level: {:?}", self.current_token))
        }
    }

    fn parse_block(&mut self) -> Result<Vec<Statement>, String> {
        let mut statements = Vec::new();
        while self.current_token != Token::RBrace && self.current_token != Token::EOF {
            statements.push(self.parse_statement()?);
        }
        self.expect(Token::RBrace)?;
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Statement, String> {
        match self.current_token {
            Token::Let => {
                self.advance();
                let name = if let Token::Identifier(n) = &self.current_token {
                    n.clone()
                } else {
                    return Err("Expected variable name".to_string());
                };
                self.advance();
                
                self.expect(Token::Colon)?;
                let ty = self.parse_type()?;
                
                self.expect(Token::Equals)?;
                let value = self.parse_expression()?;
                self.expect(Token::SemiColon)?;
                
                Ok(Statement::Let {
                    name,
                    ty,
                    value,
                    is_volatile: false, // Local volatile not shown in example, only global
                })
            }
            Token::Unsafe => {
                self.advance();
                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                Ok(Statement::UnsafeBlock(body))
            }
            Token::Loop => {
                self.advance();
                self.expect(Token::LBrace)?;
                let body = self.parse_block()?;
                Ok(Statement::LoopBlock(body))
            }
            _ => {
                // Could be assignment or expression
                let expr = self.parse_expression()?;
                if self.current_token == Token::Equals {
                    self.advance();
                    let value = self.parse_expression()?;
                    self.expect(Token::SemiColon)?;
                    Ok(Statement::Assignment { target: expr, value })
                } else {
                    self.expect(Token::SemiColon)?;
                    Ok(Statement::ExpressionStmt(expr))
                }
            }
        }
    }

    fn parse_expression(&mut self) -> Result<Expression, String> {
        self.parse_binary_expression(0)
    }

    fn parse_binary_expression(&mut self, _precedence: u8) -> Result<Expression, String> {
        // Very basic implementation, just handles | for now as in example
        let mut left = self.parse_unary()?;
        
        while self.current_token == Token::Pipe {
            self.advance();
            let right = self.parse_unary()?;
            left = Expression::BinaryOp {
                op: BinaryOperator::BitwiseOr,
                left: Box::new(left),
                right: Box::new(right),
            };
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expression, String> {
        match self.current_token {
            Token::Star => {
                self.advance();
                let expr = self.parse_unary()?;
                Ok(Expression::Dereference(Box::new(expr)))
            }
            Token::Cast => {
                self.advance();
                self.expect(Token::LessThan)?;
                let target_type = self.parse_type()?;
                self.expect(Token::GreaterThan)?;
                self.expect(Token::LParen)?;
                let value = self.parse_expression()?;
                self.expect(Token::RParen)?;
                Ok(Expression::Cast {
                    target_type,
                    value: Box::new(value),
                })
            }
            _ => self.parse_primary(),
        }
    }

    fn parse_primary(&mut self) -> Result<Expression, String> {
        match &self.current_token {
            Token::Integer(val) => {
                let v = *val;
                self.advance();
                Ok(Expression::Integer(v))
            }
            Token::Identifier(name) => {
                let n = name.clone();
                self.advance();
                if self.current_token == Token::LParen {
                    // Function call
                    self.advance();
                    let mut args = Vec::new();
                    if self.current_token != Token::RParen {
                        loop {
                            args.push(self.parse_expression()?);
                            if self.current_token == Token::Comma {
                                self.advance();
                            } else {
                                break;
                            }
                        }
                    }
                    self.expect(Token::RParen)?;
                    Ok(Expression::FunctionCall { name: n, args })
                } else {
                    Ok(Expression::Identifier(n))
                }
            }
            Token::Asm => {
                self.advance();
                self.expect(Token::LParen)?;
                if let Token::StringLiteral(s) = &self.current_token {
                    let code = s.clone();
                    self.advance();
                    self.expect(Token::RParen)?;
                    Ok(Expression::Asm(code))
                } else {
                    Err("Expected string literal for asm".to_string())
                }
            }
            _ => Err(format!("Unexpected token in expression: {:?}", self.current_token)),
        }
    }
}
