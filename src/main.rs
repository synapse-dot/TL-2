mod cst;
mod lexer;
mod parser;
mod token;

use std::{env, fs, process};

use lexer::lex;
use parser::parse;

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: tl2 [--dump-tokens|--dump-cst] <file.tlh>");
        process::exit(2);
    }

    let dump_cst = args.iter().any(|a| a == "--dump-cst");
    let dump_tokens = args.iter().any(|a| a == "--dump-tokens") || !dump_cst;
    let path = args
        .iter()
        .find(|a| !a.starts_with("--"))
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

    if dump_cst {
        match parse(&tokens) {
            Ok(tree) => println!("{:#?}", tree),
            Err(err) => {
                eprintln!(
                    "parse error at {}:{} (bytes {}..{}): {}",
                    err.span.line, err.span.column, err.span.start, err.span.end, err.message
                );
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
