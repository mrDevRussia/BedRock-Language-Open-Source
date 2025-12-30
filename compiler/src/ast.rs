#![allow(dead_code)]

#[derive(Debug, Clone)]
pub enum Type {
    U8, U16, U32, U64,
    I8, I16, I32, I64,
    F32, F64,
    Bool,
    Void,
    Pointer(Box<Type>),
}

#[derive(Debug, Clone)]
pub enum Attribute {
    Address(u64),
    Interrupt,
    Align(u64),
}

#[derive(Debug, Clone)]
pub struct GlobalVariable {
    pub name: String,
    pub ty: Type,
    pub is_volatile: bool,
    pub attributes: Vec<Attribute>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub name: String,
    pub return_type: Type,
    pub attributes: Vec<Attribute>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone)]
pub enum Statement {
    Let {
        name: String,
        ty: Type,
        value: Expression,
        is_volatile: bool, // Local volatile might not be in spec, but for completeness
    },
    UnsafeBlock(Vec<Statement>),
    LoopBlock(Vec<Statement>),
    ExpressionStmt(Expression),
    Assignment {
        target: Expression, // Can be a dereference *ptr = val
        value: Expression,
    },
}

#[derive(Debug, Clone)]
pub enum Expression {
    Integer(u64),
    Identifier(String),
    Cast {
        target_type: Type,
        value: Box<Expression>,
    },
    Dereference(Box<Expression>),
    BinaryOp {
        op: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
    FunctionCall {
        name: String,
        args: Vec<Expression>,
    },
    Asm(String),
}

#[derive(Debug, Clone)]
pub enum BinaryOperator {
    BitwiseOr,
    // Add others as needed
}

#[derive(Debug, Clone)]
pub enum TopLevelItem {
    GlobalVariable(GlobalVariable),
    Function(Function),
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<TopLevelItem>,
}
