#[derive(Debug, Clone)]
pub enum Statement {
    Let(String, Expression),
    Root(String, Expression),
    Assignment(String, Expression),
    ArrayAssign(String, Expression, Expression), 
    Loop(Vec<Statement>),
    While(Expression, Vec<Statement>),
    If(Expression, Vec<Statement>, Option<Vec<Statement>>),
    FunctionDefine(String, Vec<String>, Vec<Statement>),
    Call(String, Vec<Expression>),
    Return(Option<Expression>), 
    Asm(String),
    Outb(Expression, Expression),
    Poke(Expression, Expression),
    Break,
    CallPtr(Expression),
    ArrayDefine(String, Vec<u64>),
    StringDefine(String, String)
}

#[derive(Debug, Clone)]
pub enum Expression {
    Number(u64),
    Variable(String),
    BinaryOp(Box<Expression>, String, Box<Expression>),
    WaitKey,
    Inb(Box<Expression>),
    Peek(Box<Expression>),
    Call(String, Vec<Expression>),
    ArrayAccess(String, Box<Expression>)
}