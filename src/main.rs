mod ast;
mod cst;
mod engine;
mod lexer;
mod lower;
mod parser;
mod token;

use engine::eval::eval_program;
use engine::timeline::ConflictPolicy;
use lexer::lex;
use lower::lower_program;
use parser::parse;
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "usage: tl2 [--dump-tokens|--dump-cst] [--eval-at <ms>] [--conflict-policy lww|error] [--dump-functions] [--run] <file.tlh>"
        );
        process::exit(2);
    }

    let dump_cst = args.iter().any(|a| a == "--dump-cst");
    let dump_tokens = args.iter().any(|a| a == "--dump-tokens")
        || (!dump_cst
            && !args.iter().any(|a| a == "--eval-at")
            && !args.iter().any(|a| a == "--run"));
    let eval_at = args
        .windows(2)
        .find(|w| w[0] == "--eval-at")
        .and_then(|w| w[1].parse::<i64>().ok());
    let policy = args
        .windows(2)
        .find(|w| w[0] == "--conflict-policy")
        .map(|w| w[1].as_str())
        .map(|p| {
            if p == "error" {
                ConflictPolicy::Error
            } else {
                ConflictPolicy::LastWriteWins
            }
        })
        .unwrap_or(ConflictPolicy::LastWriteWins);
    let dump_functions = args.iter().any(|a| a == "--dump-functions");
    let run_mode = args.iter().any(|a| a == "--run");
    let path = args
        .iter()
        .find(|a| a.ends_with(".tlh"))
        .unwrap_or_else(|| {
            eprintln!("missing input file");
            process::exit(2);
        });

    let input = fs::read_to_string(path).unwrap_or_else(|err| {
        eprintln!("failed to read {path}: {err}");
        process::exit(1);
    });
    let tokens = match lex(&input) {
        Ok(tokens) => tokens,
        Err(err) => {
            eprintln!(
                "lex error at {}:{} (bytes {}..{}): {}",
                err.span.line, err.span.column, err.span.start, err.span.end, err.message
            );
            process::exit(1);
        }
    };

    let tree = if dump_cst || eval_at.is_some() || dump_functions || run_mode {
        match parse(&tokens) {
            Ok(tree) => {
                if dump_cst {
                    println!("{:#?}", tree);
                }
                Some(tree)
            }
            Err(err) => {
                eprintln!(
                    "parse error at {}:{} (bytes {}..{}): {}",
                    err.span.line, err.span.column, err.span.start, err.span.end, err.message
                );
                process::exit(1);
            }
        }
    } else {
        None
    };

    if let Some(t_ms) = eval_at {
        let cst = tree.as_ref().expect("parser tree expected");
        let program = lower_program(cst);
        match eval_program(&program, policy) {
            Ok(rt) => {
                for (name, _) in &rt.timeline.vars {
                    if let Some(v) = rt.timeline.value_at(name, t_ms) {
                        println!("{} @ {}ms = {:?}", name, t_ms, v);
                    }
                }
                if dump_functions {
                    for (name, versions) in rt.functions.all() {
                        println!("fn {} versions={}", name, versions.len());
                        for v in versions {
                            println!("  from {}ms params={:?}", v.start_ms, v.params);
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("eval error: {}", err.0);
                process::exit(1);
            }
        }
    }

    if run_mode {
        let cst = tree.as_ref().expect("parser tree expected");
        let program = lower_program(cst);
        if let Err(err) = eval_program(&program, policy) {
            eprintln!("eval error: {}", err.0);
            process::exit(1);
        }
    }
    if eval_at.is_none() && dump_functions {
        let cst = tree.as_ref().expect("parser tree expected");
        let program = lower_program(cst);
        match eval_program(&program, policy) {
            Ok(rt) => {
                for (name, versions) in rt.functions.all() {
                    println!("fn {} versions={}", name, versions.len());
                    for v in versions {
                        println!("  from {}ms params={:?}", v.start_ms, v.params);
                    }
                }
            }
            Err(err) => {
                eprintln!("eval error: {}", err.0);
                process::exit(1);
            }
        }
    }
    if dump_tokens {
        for token in tokens {
            println!(
                "{:?} @ {}:{}",
                token.kind, token.span.line, token.span.column
            );
        }
    }
}
