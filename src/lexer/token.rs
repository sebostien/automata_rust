use lazy_static::lazy_static;

use crate::{
    language::{Language, Match, self},
    nfa::{NFASet, NFA},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Spanned<T> {
    pub start: usize,
    pub token: T,
    pub end: usize,
}

pub trait Token
where
    Self: Sized,
{
    #[must_use]
    fn next_match(input: &str) -> Option<(usize, Self)> {
        // Find longest match
        let m = Self::get_token_set()
            .is_match(input)
            .into_iter()
            .max_by_key(language::Match::match_size);

        if let Some(m) = m {
            match m {
                Match::Group(label, size) => Some((size, Self::token_from_label(label.into()))),
                Match::NoGroup(_) => {
                    unreachable!("All matches from NFASet should have a group")
                }
            }
        } else {
            None
        }
    }

    #[must_use]
    fn skip_chars(input: &str) -> usize {
        Self::skip_reg()
            .is_match(input)
            .into_iter()
            .map(|m| m.match_size())
            .max()
            .unwrap_or(0)
    }

    #[must_use]
    fn skip_reg() -> &'static NFA {
        lazy_static! {
            static ref SKIP_REG: NFA = NFA::try_from_language(r"(\n|\t|\ )*").unwrap();
        }
        &SKIP_REG
    }

    #[must_use]
    fn eof() -> Option<Self>;

    #[must_use]
    fn get_skip_reg() -> &'static str;

    #[must_use]
    fn get_token_set() -> &'static NFASet;

    #[must_use]
    fn token_from_label(label: &'static str) -> Self;
}

#[macro_export]
macro_rules! impl_token {
    (
        $this:ident,
        $eof:expr,
        $(($variant:expr, $label:expr, $regex:expr)),+
    ) => {
        impl Token for $this {
            fn eof() -> Option<Self> {
                $eof
            }

            fn get_skip_reg() -> &'static str {
                r"(\n|\t|\ )*"
            }

            fn get_token_set() -> &'static NFASet {
                lazy_static! {
                    static ref TOKEN_SET: NFASet = NFASet::build(vec![
                        $(($label.into(), NFA::try_from_language($regex).unwrap())),+
                    ])
                    .unwrap();
                }
                &TOKEN_SET
            }

            fn token_from_label(label: &'static str) -> Self {
                use $this::*;
                match label {
                    $($label => $variant,)+
                    _ => unreachable!("No mapping for group: {label}"),
                }
            }
        }
    };
}
