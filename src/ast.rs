#![allow(dead_code)]

use crate::token::Span;

#[derive(Debug, Clone)]
pub enum Literal {
    Number(f64),
    String(String),
    Bool(bool),
    Null,
}

#[derive(Debug, Clone)]
pub enum TimeExpr {
    Now,
    DurationMs(i64),
}

#[derive(Debug, Clone)]
pub struct FnLiteral {
    pub params: Vec<String>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub enum Expr {
    Literal(Literal),
    Ident(String),
    Call(Box<Expr>, Vec<Expr>),
    FnLiteral(FnLiteral),
    Pid,                                      // self()
    Spawn(Box<Expr>, Vec<Expr>),             // spawn(fn, args)
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    pub init: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub name: String,
    pub params: Vec<String>,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ProcessDecl {
    pub name: String,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum RewriteTarget {
    Var(String),
    Fn(String),
}

#[derive(Debug, Clone)]
pub struct RewriteStmt {
    pub target: RewriteTarget,
    pub value: Expr,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct SendStmt {
    pub target: Expr,      // PID expression
    pub message: Expr,     // value to send
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct ReceiveStmt {
    pub pattern: String,   // variable name to bind received message
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub stmts: Vec<Stmt>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub struct AtBlock {
    pub time: TimeExpr,
    pub body: Block,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    State(StateDecl),
    Fn(FnDecl),
    Process(ProcessDecl),
    Rewrite(RewriteStmt),
    Send(SendStmt),
    Receive(ReceiveStmt),
    At(AtBlock),
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Stmt>,
}