use crate::ast::*;
use crate::cst::{
    AtBlock as CstAt, CommandStmt, CstFile, CstNode, FnDecl as CstFn, RewriteStmt as CstRewrite,
    StateDecl as CstState,
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
        CstNode::Fn(f) => lower_fn(f),
        CstNode::Rewrite(r) => lower_rewrite(r),
        CstNode::At(a) => lower_at(a),
        CstNode::Yield(c) => lower_yield(c),
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
        .and_then(|i| lower_expr_tokens(&s.tokens[i + 1..]));
    Stmt::State(StateDecl {
        name,
        init,
        span: s.span,
    })
}

fn lower_fn(f: &CstFn) -> Stmt {
    let mut it = f.header.iter();
    let _fn_kw = it.next();
    let name = match it.next().map(|t| &t.kind) {
        Some(TokenKind::Identifier(s)) => s.clone(),
        _ => "<invalid_fn>".into(),
    };

    let mut params = Vec::new();
    let mut in_params = false;
    for t in &f.header {
        match &t.kind {
            TokenKind::LParen => in_params = true,
            TokenKind::RParen => in_params = false,
            TokenKind::Identifier(s) if in_params => params.push(s.clone()),
            _ => {}
        }
    }

    Stmt::Fn(FnDecl {
        name,
        params,
        body: Block {
            stmts: f.body.items.iter().map(lower_node).collect(),
            span: f.body.span,
        },
        span: f.span,
    })
}

fn lower_rewrite(r: &CstRewrite) -> Stmt {
    let target_name = match r.tokens.get(1).map(|t| &t.kind) {
        Some(TokenKind::Identifier(id)) => id.clone(),
        _ => "<invalid>".to_string(),
    };
    let target = RewriteTarget::Fn(target_name);

    let value = if let Some(i) = r
        .tokens
        .iter()
        .position(|t| t.kind == TokenKind::FatArrow || t.kind == TokenKind::Eq)
    {
        if matches!(
            r.tokens.get(i + 1).map(|t| &t.kind),
            Some(TokenKind::LBrace)
        ) {
            let body_tokens = collect_brace_block(&r.tokens[i + 1..]);
            let yield_expr =
                extract_yield_expr(&body_tokens).unwrap_or(Expr::Literal(Literal::Null));
            Expr::FnLiteral(FnLiteral {
                params: Vec::new(),
                body: Block {
                    stmts: vec![Stmt::Yield(yield_expr)],
                    span: body_tokens.last().map(|t| t.span).unwrap_or(r.span),
                },
            })
        } else {
            lower_expr_tokens(&r.tokens[i + 1..]).unwrap_or(Expr::Literal(Literal::Null))
        }
    } else {
        Expr::Literal(Literal::Null)
    };

    Stmt::Rewrite(RewriteStmt {
        target,
        value,
        span: r.span,
    })
}

fn lower_yield(c: &CommandStmt) -> Stmt {
    let expr = lower_expr_tokens(&c.tokens[1..]).unwrap_or(Expr::Literal(Literal::Null));
    Stmt::Yield(expr)
}

fn extract_yield_expr(tokens: &[Token]) -> Option<Expr> {
    let i = tokens
        .iter()
        .position(|t| matches!(t.kind, TokenKind::Keyword(Keyword::Yield)))?;
    lower_expr_tokens(tokens.get(i + 1..)?)
}

fn collect_brace_block(tokens: &[Token]) -> Vec<Token> {
    let mut out = Vec::new();
    let mut d = 0usize;
    for t in tokens {
        out.push(t.clone());
        match t.kind {
            TokenKind::LBrace => d += 1,
            TokenKind::RBrace => {
                d = d.saturating_sub(1);
                if d == 0 {
                    break;
                }
            }
            _ => {}
        }
    }
    out
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

fn lower_expr_tokens(tokens: &[Token]) -> Option<Expr> {
    if tokens.len() >= 3 {
        if let (TokenKind::Identifier(name), TokenKind::LParen, TokenKind::RParen) =
            (&tokens[0].kind, &tokens[1].kind, &tokens[2].kind)
        {
            return Some(Expr::Call(Box::new(Expr::Ident(name.clone())), Vec::new()));
        }
        if let (TokenKind::Identifier(name), TokenKind::LParen) = (&tokens[0].kind, &tokens[1].kind)
        {
            let mut args = Vec::new();
            let mut i = 2usize;
            while i < tokens.len() {
                if matches!(tokens[i].kind, TokenKind::RParen) {
                    break;
                }
                if !matches!(tokens[i].kind, TokenKind::Comma) {
                    if let Some(arg) = lower_expr_token(&tokens[i]) {
                        args.push(arg);
                    }
                }
                i += 1;
            }
            return Some(Expr::Call(Box::new(Expr::Ident(name.clone())), args));
        }
    }
    tokens.first().and_then(lower_expr_token)
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
