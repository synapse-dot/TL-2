use crate::ast::*;
use crate::cst::{
    AtBlock as CstAt, CstFile, CstNode, RewriteStmt as CstRewrite, StateDecl as CstState,
};
use crate::token::{Keyword, TimeUnit, Token, TokenKind};

pub fn lower_program(cst: &CstFile) -> Program {
    Program {
        items: cst.items.iter().map(lower_node).collect(),
    }
}

fn lower_node(node: &CstNode) -> Stmt {
    match node {
        CstNode::State(s) => lower_state(s),
        CstNode::Rewrite(r) => lower_rewrite(r),
        CstNode::At(a) => lower_at(a),
        _ => Stmt::Unsupported,
    }
}

fn lower_state(s: &CstState) -> Stmt {
    let name = match s.tokens.get(1).map(|t| &t.kind) {
        Some(TokenKind::Identifier(id)) => id.clone(),
        _ => "<invalid>".to_string(),
    };
    let init = s
        .tokens
        .iter()
        .position(|t| t.kind == TokenKind::Eq)
        .and_then(|i| s.tokens.get(i + 1))
        .and_then(lower_expr_token);

    Stmt::State(StateDecl {
        name,
        init,
        span: s.span,
    })
}

fn lower_rewrite(r: &CstRewrite) -> Stmt {
    let target = match r.tokens.get(1).map(|t| &t.kind) {
        Some(TokenKind::Identifier(id)) => RewriteTarget::Var(id.clone()),
        _ => RewriteTarget::Var("<invalid>".to_string()),
    };

    let value = r
        .tokens
        .iter()
        .position(|t| t.kind == TokenKind::FatArrow || t.kind == TokenKind::Eq)
        .and_then(|i| r.tokens.get(i + 1))
        .and_then(lower_expr_token)
        .unwrap_or(Expr::Literal(Literal::Null));

    Stmt::Rewrite(RewriteStmt {
        target,
        value,
        span: r.span,
    })
}

fn lower_at(a: &CstAt) -> Stmt {
    let time = parse_time_expr(&a.header).unwrap_or(TimeExpr::DurationMs(0));
    let stmts = a.body.items.iter().map(lower_node).collect();
    Stmt::At(AtBlock {
        time,
        body: Block {
            stmts,
            span: a.body.span,
        },
        span: a.span,
    })
}

fn parse_time_expr(tokens: &[Token]) -> Option<TimeExpr> {
    if tokens
        .iter()
        .any(|t| matches!(t.kind, TokenKind::Keyword(Keyword::Now)))
    {
        return Some(TimeExpr::Now);
    }

    let t = tokens
        .iter()
        .find(|t| matches!(t.kind, TokenKind::TimeNumber { .. }))?;
    if let TokenKind::TimeNumber { value, unit } = &t.kind {
        let n = value.parse::<f64>().ok()?;
        let ms = match unit {
            TimeUnit::Ms => n,
            TimeUnit::S => n * 1000.0,
            TimeUnit::Min => n * 60_000.0,
            TimeUnit::H => n * 3_600_000.0,
            TimeUnit::D => n * 86_400_000.0,
        };
        return Some(TimeExpr::DurationMs(ms as i64));
    }
    None
}

fn lower_expr_token(t: &Token) -> Option<Expr> {
    match &t.kind {
        TokenKind::Number(n) => n
            .parse::<f64>()
            .ok()
            .map(|x| Expr::Literal(Literal::Number(x))),
        TokenKind::StringLiteral(s) => Some(Expr::Literal(Literal::String(s.clone()))),
        TokenKind::Identifier(i) => Some(Expr::Ident(i.clone())),
        TokenKind::Keyword(Keyword::True) => Some(Expr::Literal(Literal::Bool(true))),
        TokenKind::Keyword(Keyword::False) => Some(Expr::Literal(Literal::Bool(false))),
        TokenKind::Keyword(Keyword::Null) => Some(Expr::Literal(Literal::Null)),
        _ => None,
    }
}
