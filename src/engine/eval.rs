use crate::cst::{AtBlock, CstFile, CstNode, RewriteStmt, StateDecl};
use crate::engine::timeline::{ConflictPolicy, TimelineStore, Value};
use crate::token::{Keyword, TimeUnit, Token, TokenKind};

#[derive(Debug)]
pub struct EvalError(pub String);

pub fn eval_program(cst: &CstFile, policy: ConflictPolicy) -> Result<TimelineStore, EvalError> {
    let mut store = TimelineStore::default();
    eval_nodes(&cst.items, 0, &mut store, policy)?;
    Ok(store)
}

fn eval_nodes(
    nodes: &[CstNode],
    base_time: i64,
    store: &mut TimelineStore,
    policy: ConflictPolicy,
) -> Result<(), EvalError> {
    for node in nodes {
        match node {
            CstNode::State(s) => eval_state(s, base_time, store, policy)?,
            CstNode::Rewrite(r) => eval_rewrite(r, base_time, store, policy)?,
            CstNode::At(a) => eval_at(a, base_time, store, policy)?,
            _ => {}
        }
    }
    Ok(())
}

fn eval_state(
    s: &StateDecl,
    at_ms: i64,
    store: &mut TimelineStore,
    policy: ConflictPolicy,
) -> Result<(), EvalError> {
    if let Some((name, value)) = parse_assignment(&s.tokens) {
        store
            .set_from(&name, at_ms, value, policy)
            .map_err(EvalError)?;
    }
    Ok(())
}

fn eval_rewrite(
    r: &RewriteStmt,
    at_ms: i64,
    store: &mut TimelineStore,
    policy: ConflictPolicy,
) -> Result<(), EvalError> {
    if let Some((name, value)) = parse_rewrite(&r.tokens) {
        store
            .set_from(&name, at_ms, value, policy)
            .map_err(EvalError)?;
    }
    Ok(())
}

fn eval_at(
    a: &AtBlock,
    base_time: i64,
    store: &mut TimelineStore,
    policy: ConflictPolicy,
) -> Result<(), EvalError> {
    let dt = parse_time_offset(&a.header).unwrap_or(0);
    eval_nodes(&a.body.items, base_time + dt, store, policy)
}

fn parse_assignment(tokens: &[Token]) -> Option<(String, Value)> {
    if tokens.len() < 4 {
        return None;
    }
    if !matches!(tokens.first()?.kind, TokenKind::Keyword(Keyword::State)) {
        return None;
    }
    let name = match &tokens.get(1)?.kind {
        TokenKind::Identifier(s) => s.clone(),
        _ => return None,
    };
    let eq_pos = tokens.iter().position(|t| t.kind == TokenKind::Eq)?;
    let value = parse_value(tokens.get(eq_pos + 1)?)?;
    Some((name, value))
}

fn parse_rewrite(tokens: &[Token]) -> Option<(String, Value)> {
    if tokens.len() < 4 {
        return None;
    }
    if !matches!(tokens.first()?.kind, TokenKind::Keyword(Keyword::Rewrite)) {
        return None;
    }
    let name = match &tokens.get(1)?.kind {
        TokenKind::Identifier(s) => s.clone(),
        _ => return None,
    };
    let arrow_pos = tokens
        .iter()
        .position(|t| t.kind == TokenKind::FatArrow || t.kind == TokenKind::Eq)?;
    let value = parse_value(tokens.get(arrow_pos + 1)?)?;
    Some((name, value))
}

fn parse_time_offset(tokens: &[Token]) -> Option<i64> {
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
        return Some(ms as i64);
    }
    None
}

fn parse_value(t: &Token) -> Option<Value> {
    match &t.kind {
        TokenKind::Number(n) => n.parse::<f64>().ok().map(Value::Number),
        TokenKind::StringLiteral(s) => Some(Value::Str(s.clone())),
        TokenKind::Keyword(Keyword::True) => Some(Value::Bool(true)),
        TokenKind::Keyword(Keyword::False) => Some(Value::Bool(false)),
        TokenKind::Keyword(Keyword::Null) => Some(Value::Null),
        _ => None,
    }
}
