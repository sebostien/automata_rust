use std::process::ExitCode;

use automata_rust::lexer::prelude::*;
use lazy_static::lazy_static;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExprToken {
    Var,
    Op,
    Num,
}

impl_token!(
    ExprToken,
    None,
    (Var, "var", r"(a-z|A-z)(a-z|A-Z|0-9)*"),
    (Op, "op", r"\+|\-"),
    (Num, "num", r"(0-9)+")
);

fn main() -> ExitCode {
    let text = std::env::args().skip(1);

    let input = text.collect::<Vec<_>>().join(" ");

    if input.is_empty() {
        eprintln!("Please provide some input!\nFor example: '2 + 2 - 3'");
        ExitCode::FAILURE
    } else {
        let lexer = Lexer::<ExprToken>::new(&input);
        let tokens = lexer.into_iter().collect::<Result<Vec<_>, _>>();

        match tokens {
            Ok(tokens) => {
                println!("{:#?}", tokens);
                ExitCode::SUCCESS
            }
            Err(e) => match e {
                LexError::UnrecognizedToken(loc) => {
                    eprintln!("Unrecognized token '{}'", &input[loc..=loc]);
                    ExitCode::FAILURE
                }
            },
        }
    }
}
