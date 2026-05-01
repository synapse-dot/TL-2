mod lexer;
mod token;

use std::{env, fs, process};

use lexer::lex;

fn main() {
    let path = env::args().nth(1).unwrap_or_else(|| {
        eprintln!("usage: tl2 <file.tlh>");
        process::exit(2);
    });

    let input = fs::read_to_string(&path).unwrap_or_else(|err| {
        eprintln!("failed to read {path}: {err}");
        process::exit(1);
    });

    match lex(&input) {
        Ok(tokens) => {
            for token in tokens {
                println!(
                    "{:?} @ {}:{}",
                    token.kind, token.span.line, token.span.column
                );
            }
        }
        Err(err) => {
            eprintln!(
                "lex error at {}:{} (bytes {}..{}): {}",
                err.span.line, err.span.column, err.span.start, err.span.end, err.message
            );
            process::exit(1);
        }
    }
}
