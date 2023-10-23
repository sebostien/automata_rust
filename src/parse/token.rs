use super::Lit;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Token {
    /// Matches the end of input '$'
    Eof,
    /// Opening parenthesis '('
    OParen,
    /// Closing parenthesis ')'
    CParen,
    /// Kleene Star '*'
    KleeneS,
    /// Kleene Plus '+'
    KleeneP,
    /// Concatenation (implicit)
    Concat,
    /// Union '|'
    Union,
    /// Optional '?'
    Optional,
    /// Range '-'
    Range,
    /// Singelton and group
    Lit(Lit),
}

impl Token {
    #[must_use]
    pub fn infix_precedence(&self) -> Option<(usize, usize)> {
        match self {
            Self::Range => Some((12, 11)),
            Self::Concat => Some((4, 3)),
            Self::Union => Some((2, 1)),
            _ => None,
        }
    }

    #[must_use]
    pub fn postfix_precedence(&self) -> Option<usize> {
        match self {
            Self::KleeneP | Self::KleeneS => Some(10),
            Self::Optional => Some(9),
            _ => None,
        }
    }
}

impl std::fmt::Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OParen => "(".fmt(f),
            Self::CParen => ")".fmt(f),
            Self::KleeneS => "*".fmt(f),
            Self::KleeneP => "+".fmt(f),
            Self::Concat => "".fmt(f),
            Self::Union => "|".fmt(f),
            Self::Optional => "?".fmt(f),
            Self::Range => "-".fmt(f),
            Self::Eof => "$".fmt(f),
            Self::Lit(c) => c.fmt(f),
        }
    }
}
