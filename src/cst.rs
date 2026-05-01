#![allow(dead_code)]

use crate::token::{Span, Token};

#[derive(Debug, Clone)]
pub struct CstFile {
    pub items: Vec<CstNode>,
}

#[derive(Debug, Clone)]
pub enum CstNode {
    State(StateDecl),
    Fn(FnDecl),
    Process(ProcessDecl),
    At(AtBlock),
    Dock(DockBlock),
    Rewrite(RewriteStmt),
    Commit(CommitStmt),
    Send(CommandStmt),
    Receive(CommandStmt),
    Yield(CommandStmt),
    Observe(CommandStmt),
    Morph(CommandStmt),
    Spawn(CommandStmt),
    Grant(CommandStmt),
    Revoke(CommandStmt),
    Expr(ExprStmt),
}

#[derive(Debug, Clone)]
pub struct StateDecl {
    pub span: Span,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct FnDecl {
    pub span: Span,
    pub header: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct ProcessDecl {
    pub span: Span,
    pub header: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct AtBlock {
    pub span: Span,
    pub header: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct DockBlock {
    pub span: Span,
    pub header: Vec<Token>,
    pub body: Block,
}

#[derive(Debug, Clone)]
pub struct RewriteStmt {
    pub span: Span,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct CommitStmt {
    pub span: Span,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct ExprStmt {
    pub span: Span,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct CommandStmt {
    pub span: Span,
    pub tokens: Vec<Token>,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub span: Span,
    pub items: Vec<CstNode>,
}
