use crate::parse::{ParseError, Token};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileError {
    /// Did not expect an empty stack when handling token.
    EmptyStack {
        token: Token,
    },
    /// Expected a stack of size 1 but found stack with `size`.
    NonUnaryStack {
        size: usize,
    },
    UnexpectedOpenParen,
    UnexpectedCloseParen,
    UnexpectedRange,
    ParseError(String),
}

impl std::fmt::Display for CompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyStack { token } => {
                writeln!(f, "Empty stack when handling token '{token}'")
            }
            Self::NonUnaryStack { size } => {
                writeln!(f, "Expected stack of size 1 but the stack had size {size}")
            }
            Self::UnexpectedOpenParen => writeln!(f, "Unexpected '('"),
            Self::UnexpectedCloseParen => writeln!(f, "Unexpected ')'"),
            Self::UnexpectedRange => writeln!(f, "Unexpected '-'"),
            Self::ParseError(s) => writeln!(f, "Parse error: {s}"),
        }
    }
}

impl std::error::Error for CompileError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LanguageError {
    CompileError(CompileError),
    ParseError(ParseError),
}

impl std::fmt::Display for LanguageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CompileError(e) => e.fmt(f),
            Self::ParseError(e) => e.fmt(f),
        }
    }
}

impl std::error::Error for LanguageError {}

impl From<CompileError> for LanguageError {
    fn from(e: CompileError) -> Self {
        Self::CompileError(e)
    }
}

impl From<ParseError> for LanguageError {
    fn from(e: ParseError) -> Self {
        Self::ParseError(e)
    }
}

pub trait Language: Sized {
    /// Check if `input` is accepted by the regex.
    /// Returns the length of the match from the start, or `None` if no match was found.
    ///
    /// The match will always start on the first char in the input.
    #[must_use]
    fn is_match(&self, input: &str) -> Vec<Match>;

    /// Convert the language to a string.
    #[must_use]
    fn to_language(&self) -> String;

    /// Parse a language string.
    #[must_use]
    fn try_from_language<S: AsRef<str>>(source: S) -> Result<Self, LanguageError>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Label(&'static str);

impl From<Label> for &'static str {
    fn from(value: Label) -> Self {
        value.0
    }
}

impl From<&'static str> for Label {
    fn from(value: &'static str) -> Self {
        Self(value)
    }
}

impl std::fmt::Display for Label {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Match {
    /// Match from group
    Group(Label, usize),
    /// Match without group
    NoGroup(usize),
}

impl Match {
    #[must_use]
    pub fn match_size(&self) -> usize {
        match *self {
            Self::Group(_, s) | Self::NoGroup(s) => s,
        }
    }
}
