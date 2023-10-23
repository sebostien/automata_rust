use std::ops::RangeInclusive;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Lit {
    Char(char),
    Any,
    Range(RangeInclusive<char>),
}

impl Lit {
    #[must_use]
    pub fn accepts(&self, c: char) -> bool {
        match self {
            &Self::Char(l) => l == c,
            Self::Any => true,
            Self::Range(r) => r.contains(&c),
        }
    }
}

impl std::fmt::Display for Lit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Any => ".".fmt(f),
            Self::Char(c) => {
                if matches!(c, '+' | '-' | '*' | '?' | '(' | ')') {
                    write!(f, r"\{c}")
                } else {
                    c.escape_default().fmt(f)
                }
            }
            Self::Range(r) => write!(f, "({}-{})", r.start(), r.end()),
        }
    }
}
