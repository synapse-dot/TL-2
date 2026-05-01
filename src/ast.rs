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
pub enum Expr {
    Literal(Literal),
    Ident(String),
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub name: String,
    pub init: Option<Expr>,
    pub span: Span,
}

#[derive(Debug, Clone)]
pub enum RewriteTarget {
    Var(String),
}

#[derive(Debug, Clone)]
pub struct RewriteStmt {
    pub target: RewriteTarget,
    pub value: Expr,
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
    Rewrite(RewriteStmt),
    At(AtBlock),
    Unsupported,
}

#[derive(Debug, Clone)]
pub struct Program {
    pub items: Vec<Stmt>,
}
