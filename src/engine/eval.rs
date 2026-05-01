use std::collections::HashMap;

use crate::ast::{Expr, Literal, Program, RewriteTarget, Stmt, TimeExpr};
use crate::engine::function_store::FunctionStore;
use crate::engine::timeline::{ConflictPolicy, TimelineStore, Value};

#[derive(Debug)]
pub struct EvalError(pub String);

#[derive(Debug, Default)]
pub struct Runtime {
    pub timeline: TimelineStore,
    pub functions: FunctionStore,
}

pub fn eval_program(program: &Program, policy: ConflictPolicy) -> Result<Runtime, EvalError> {
    let mut rt = Runtime::default();
    eval_stmts(&program.items, 0, &mut rt, policy, &mut HashMap::new())?;
    Ok(rt)
}

fn eval_stmts(
    stmts: &[Stmt],
    current_ms: i64,
    rt: &mut Runtime,
    policy: ConflictPolicy,
    scope: &mut HashMap<String, Value>,
) -> Result<Option<Value>, EvalError> {
    for stmt in stmts {
        match stmt {
            Stmt::State(s) => {
                if let Some(expr) = &s.init {
                    let v = eval_expr(expr, current_ms, rt, scope)?;
                    rt.timeline
                        .set_from(&s.name, current_ms, v, policy)
                        .map_err(EvalError)?;
                }
            }
            Stmt::Rewrite(r) => match &r.target {
                RewriteTarget::Var(name) => {
                    let v = eval_expr(&r.value, current_ms, rt, scope)?;
                    rt.timeline
                        .set_from(name, current_ms, v, policy)
                        .map_err(EvalError)?;
                }
                RewriteTarget::Fn(name) => {
                    if let Expr::FnLiteral(f) = &r.value {
                        rt.functions.define(
                            name.clone(),
                            current_ms,
                            f.params.clone(),
                            f.body.clone(),
                        );
                    }
                }
            },
            Stmt::Let(name, expr) => {
                let v = eval_expr(expr, current_ms, rt, scope)?;
                scope.insert(name.clone(), v);
            }
            Stmt::Expr(expr) => {
                let _ = eval_expr(expr, current_ms, rt, scope)?;
            }
            Stmt::Fn(f) => {
                rt.functions
                    .define(f.name.clone(), current_ms, f.params.clone(), f.body.clone());
            }
            Stmt::At(at) => {
                let next = match at.time {
                    TimeExpr::Now => current_ms,
                    TimeExpr::DurationMs(dt) => current_ms + dt,
                };
                if let Some(v) = eval_stmts(&at.body.stmts, next, rt, policy, scope)? {
                    return Ok(Some(v));
                }
            }
            Stmt::Yield(expr) => {
                let v = eval_expr(expr, current_ms, rt, scope)?;
                return Ok(Some(v));
            }
            Stmt::Unsupported => {}
        }
    }
    Ok(None)
}

fn eval_expr(
    expr: &Expr,
    current_ms: i64,
    rt: &mut Runtime,
    scope: &HashMap<String, Value>,
) -> Result<Value, EvalError> {
    match expr {
        Expr::Literal(Literal::Number(n)) => Ok(Value::Number(*n)),
        Expr::Literal(Literal::String(s)) => Ok(Value::Str(s.clone())),
        Expr::Literal(Literal::Bool(b)) => Ok(Value::Bool(*b)),
        Expr::Literal(Literal::Null) => Ok(Value::Null),
        Expr::Ident(name) => {
            if let Some(v) = scope.get(name) {
                return Ok(v.clone());
            }
            rt.timeline
                .value_at(name, current_ms)
                .cloned()
                .ok_or_else(|| EvalError(format!("unknown identifier: {name}")))
        }
        Expr::Call(callee, args) => {
            if let Expr::Ident(name) = &**callee {
                if name == "debug" {
                    let arg = args
                        .first()
                        .ok_or_else(|| EvalError("debug() needs one argument".into()))?;
                    let v = eval_expr(arg, current_ms, rt, scope)?;
                    println!("debug @ {}ms: {:?}", current_ms, v);
                    return Ok(Value::Null);
                }
                let f = rt
                    .functions
                    .active_at(name, current_ms)
                    .cloned()
                    .ok_or_else(|| EvalError(format!("unknown function: {name}")))?;
                let mut local = HashMap::new();
                for (idx, p) in f.params.iter().enumerate() {
                    if let Some(a) = args.get(idx) {
                        local.insert(p.clone(), eval_expr(a, current_ms, rt, scope)?);
                    }
                }
                if let Some(v) = eval_stmts(
                    &f.body.stmts,
                    current_ms,
                    rt,
                    ConflictPolicy::LastWriteWins,
                    &mut local,
                )? {
                    return Ok(v);
                }
                return Ok(Value::Null);
            }
            Ok(Value::Null)
        }
        Expr::FnLiteral(_) => Ok(Value::Null),
    }
}
