mod cst;
mod engine;
mod lexer;
mod parser;
mod token;

use std::{env, fs, process};

use engine::eval::eval_program;
use engine::timeline::ConflictPolicy;
use lexer::lex;
use parser::parse;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!(
            "usage: tl2 [--dump-tokens|--dump-cst] [--eval-at <ms>] [--conflict-policy lww|error] <file.tlh>"
        );
        process::exit(2);
    }

    let dump_cst = args.iter().any(|a| a == "--dump-cst");
    let dump_tokens = args.iter().any(|a| a == "--dump-tokens")
        || (!dump_cst && !args.iter().any(|a| a == "--eval-at"));
    let eval_at = args
        .windows(2)
        .find(|w| w[0] == "--eval-at")
        .and_then(|w| w[1].parse::<i64>().ok());

    let policy = args
        .windows(2)
        .find(|w| w[0] == "--conflict-policy")
        .map(|w| w[1].as_str())
        .map(|p| match p {
            "error" => ConflictPolicy::Error,
            _ => ConflictPolicy::LastWriteWins,
        })
        .unwrap_or(ConflictPolicy::LastWriteWins);
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

    let tree = if dump_cst || eval_at.is_some() {
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
        let program = tree.as_ref().expect("parser tree expected");
        match eval_program(program, policy) {
            Ok(store) => {
                for (name, _) in &store.vars {
                    if let Some(v) = store.value_at(name, t_ms) {
                        println!("{} @ {}ms = {:?}", name, t_ms, v);
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
