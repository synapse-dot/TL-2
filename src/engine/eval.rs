use crate::ast::{Expr, Literal, Program, RewriteTarget, Stmt, TimeExpr};
use crate::engine::timeline::{ConflictPolicy, TimelineStore, Value};

#[derive(Debug)]
pub struct EvalError(pub String);

pub fn eval_program(program: &Program, policy: ConflictPolicy) -> Result<TimelineStore, EvalError> {
    let mut store = TimelineStore::default();
    eval_stmts(&program.items, 0, &mut store, policy)?;
    Ok(store)
}

fn eval_stmts(
    stmts: &[Stmt],
    current_ms: i64,
    store: &mut TimelineStore,
    policy: ConflictPolicy,
) -> Result<(), EvalError> {
    for stmt in stmts {
        match stmt {
            Stmt::State(s) => {
                if let Some(expr) = &s.init {
                    let v = eval_expr(expr, current_ms, store)?;
                    store
                        .set_from(&s.name, current_ms, v, policy)
                        .map_err(EvalError)?;
                }
            }
            Stmt::Rewrite(r) => {
                let name = match &r.target {
                    RewriteTarget::Var(v) => v,
                };
                let v = eval_expr(&r.value, current_ms, store)?;
                store
                    .set_from(name, current_ms, v, policy)
                    .map_err(EvalError)?;
            }
            Stmt::At(at) => {
                let next = match at.time {
                    TimeExpr::Now => current_ms,
                    TimeExpr::DurationMs(dt) => current_ms + dt,
                };
                eval_stmts(&at.body.stmts, next, store, policy)?;
            }
            Stmt::Unsupported => {}
        }
    }
    Ok(())
}

fn eval_expr(expr: &Expr, current_ms: i64, store: &TimelineStore) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(Literal::Number(n)) => Ok(Value::Number(*n)),
        Expr::Literal(Literal::String(s)) => Ok(Value::Str(s.clone())),
        Expr::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
        Expr::Literal(Literal::Null) => Ok(Value::Null),
        Expr::Ident(name) => store
            .value_at(name, current_ms)
            .cloned()
            .ok_or_else(|| EvalError(format!("unknown identifier: {name}"))),
    }
}
