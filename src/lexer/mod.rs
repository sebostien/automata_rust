use std::marker::PhantomData;

use self::token::{Spanned, Token};

pub mod token;

pub mod prelude {
    pub use super::token::{Spanned, Token};
    pub use super::{LexError, Lexer};

    pub use crate::impl_token;
    pub use crate::language::Language;
    pub use crate::nfa::{NFASet, NFA};
}

#[derive(Debug)]
pub struct Lexer<'input, T> {
    input: &'input str,
    consumed: usize,
    phantom: PhantomData<T>,
    /// True when the input is empty and a `T::eof()` token has been returned.
    sent_eof: bool,
    /// True when an error has been found and we could not skip forward in the input stream.
    /// When this is `true` the iterator only produces `None`.
    sent_error: bool,
}

impl<'input, T> Lexer<'input, T> {
    #[must_use]
    pub fn new(input: &'input str) -> Self {
        Self {
            input,
            consumed: 0,
            phantom: PhantomData,
            sent_eof: false,
            sent_error: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LexError {
    UnrecognizedToken(usize),
}

impl std::fmt::Display for LexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::UnrecognizedToken(loc) => write!(f, "Unrecognized token at {loc}"),
        }
    }
}

impl From<LexError> for String {
    fn from(value: LexError) -> Self {
        value.to_string()
    }
}

impl std::error::Error for LexError {}

impl<'input, T: Token> Iterator for Lexer<'input, T>
where
    T: std::fmt::Debug,
{
    type Item = Result<Spanned<T>, LexError>;

    fn next(&mut self) -> Option<Self::Item> {
        let skipped = T::skip_chars(self.input);
        self.input = &self.input[skipped..];
        self.consumed += skipped;

        if self.sent_error || self.sent_eof {
            return None;
        }

        if self.input.is_empty() {
            self.sent_eof = true;
            return T::eof().map(|t| {
                Ok(Spanned {
                    start: self.consumed,
                    token: t,
                    end: self.consumed,
                })
            });
        }

        let token = T::next_match(self.input)
            .map(|(consumed, token)| {
                let start = self.consumed;
                self.consumed += consumed;
                self.input = &self.input[consumed..];
                Spanned {
                    start,
                    token,
                    end: self.consumed,
                }
            })
            .ok_or_else(|| {
                let consumed = self.consumed;
                // We try to skip one char and continue.
                if let Some(c) = self.input.chars().next() {
                    self.input = &self.input[c.len_utf8()..];
                    self.consumed += c.len_utf8();
                } else {
                    // We end the iterator if we can't skip
                    self.sent_error = true;
                }
                LexError::UnrecognizedToken(consumed)
            });

        Some(token)
    }
}

#[cfg(test)]
pub mod tests {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum ExprToken {
        Var,
        Op,
        Num,
    }

    use super::prelude::*;
    use lazy_static::lazy_static;

    impl_token!(
        ExprToken,
        None,
        (Var, "var", r"(a-z|A-z)(a-z|A-Z|0-9)*"),
        (Op, "op", r"\+|\-"),
        (Num, "num", r"(0-9)+")
    );

    #[test]
    fn lexer() {
        // crate::graph_display::print_nfa_svg(&REG_SET.0);
        let input = "one1+two2 - 1 +21 a20";

        let lexer = Lexer::<ExprToken>::new(input);
        let tokens = lexer
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap()
            .into_iter()
            .map(|Spanned { token, .. }| token)
            .collect::<Vec<_>>();

        use ExprToken::*;
        assert_eq!(tokens, vec![Var, Op, Var, Op, Num, Op, Num, Var],);

        // Invalid '/'
        let input = "zx + yx - xx * (y / x)";
        let lexer = Lexer::<ExprToken>::new(input);
        let tokens = lexer.into_iter().collect::<Result<Vec<_>, _>>();
        assert!(tokens.is_err());

        // Unrecognized '/' and '!'
        let input = "-2 + 4 + -2 + 2 / 2 !";
        let lexer = Lexer::<ExprToken>::new(input);
        let tokens = lexer
            .into_iter()
            .filter_map(|res| match res {
                Ok(_) => None,
                Err(err) => Some(err),
            })
            .collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                LexError::UnrecognizedToken(16),
                LexError::UnrecognizedToken(20)
            ]
        );
    }
}
