use std::collections::VecDeque;
use std::iter::Peekable;
use std::str::Chars;

mod lit;
mod postfix;
mod token;

pub use lit::Lit;
pub use postfix::Postfix;
pub use token::Token;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    Unmatched(&'static str),
    ParsingStopped(Token),
    InvalidPrefix(Token),
    InvalidRange {
        found: String,
        expected: &'static str,
    },
    UnexpectedEof,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ParsingStopped(token) => write!(f, "Parsing stopped at token: `{token}`"),
            Self::Unmatched(s) => write!(f, "Unmatched '{s}'"),
            Self::InvalidPrefix(s) => write!(f, "Token '{s}' cannot appear as a prefix"),
            Self::InvalidRange { found, expected } => write!(
                f,
                "Invalid group: Expected token '{expected}' but found '{found}'"
            ),
            Self::UnexpectedEof => "Unexpected EOF".fmt(f),
        }
    }
}

impl std::error::Error for ParseError {}

#[derive(Debug)]
struct Lexer<'i> {
    input: Peekable<Chars<'i>>,
    queue: VecDeque<Token>,
}

impl<'i> Lexer<'i> {
    #[must_use]
    fn new(input: &'i str) -> Self {
        Self {
            input: input.chars().peekable(),
            queue: VecDeque::new(),
        }
    }

    #[must_use]
    fn peek(&mut self) -> Option<&Token> {
        if self.queue.front().is_some() {
            return self.queue.front();
        }

        while let Some(next) = self.input.next() {
            if next.is_whitespace() {
                continue;
            }

            // True if we need to insert an implicit concatenation into the token stream
            let mut needs_concat = true;
            let next = match next {
                '(' => {
                    needs_concat = false;
                    Token::OParen
                }
                '|' => {
                    needs_concat = false;
                    Token::Union
                }
                '-' => {
                    needs_concat = false;
                    Token::Range
                }
                ')' => Token::CParen,
                '*' => Token::KleeneS,
                '+' => Token::KleeneP,
                '?' => Token::Optional,
                '$' => {
                    needs_concat = false;
                    Token::Eof
                }
                '\\' => {
                    if let Some(c) = self.input.next() {
                        // TODO: Might be more than these...
                        let lit = match c {
                            'n' => Lit::Char('\n'),
                            't' => Lit::Char('\t'),
                            'r' => Lit::Char('\r'),
                            _ => Lit::Char(c),
                        };
                        Token::Lit(lit)
                    } else {
                        panic!("Unexpected Eof");
                    }
                }
                c => Token::Lit(Lit::Char(c)),
            };

            if needs_concat {
                while let Some(c) = self.input.peek() {
                    if c.is_whitespace() {
                        self.input.next();
                        continue;
                    }

                    if !matches!(c, ')' | '*' | '+' | '|' | '?' | '-') {
                        self.queue.push_back(Token::Concat);
                    }

                    break;
                }
            }

            self.queue.push_front(next);
            return self.queue.front();
        }

        None
    }
}

impl Iterator for Lexer<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(p) = self.queue.pop_front() {
            Some(p)
        } else if self.peek().is_some() {
            // Peek inserts token into queue so next time we won't get here
            self.next()
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse() {
        // assert_eq!("A".parse::<Postfix>().unwrap().to_string(), "A");
        assert_eq!(
            r#" \nA\t
                   "#
            .parse::<Postfix>()
            .unwrap()
            .to_string(),
            r"(\n(A\t))"
        );
        assert_eq!(
            "A? B|C".parse::<Postfix>().unwrap().to_string(),
            "(((A?)B)|C)"
        );
        assert_eq!(
            "AB|((A|C) B|C?)".parse::<Postfix>().unwrap().to_string(),
            "((AB)|(((A|C)B)|(C?)))"
        );
        assert_eq!(
            "A? | B* +".parse::<Postfix>().unwrap().to_string(),
            "((A?)|((B*)+))"
        );
        assert_eq!(
            "((((( (A) )))?))".parse::<Postfix>().unwrap().to_string(),
            "(A?)"
        );
        assert_eq!(
            "(AC?) (B|C?A)".parse::<Postfix>().unwrap().to_string(),
            "((A(C?))(B|((C?)A)))"
        );
        assert_eq!(
            "(A-Z|a-z)(A-Za-z0-9)*"
                .parse::<Postfix>()
                .unwrap()
                .to_string(),
            "(((A-Z)|(a-z))(((A-Z)((a-z)(0-9)))*))"
        );

        assert!("A|(B?".parse::<Postfix>().is_err());
        assert!("A)|B?".parse::<Postfix>().is_err());
        assert!("A|?".parse::<Postfix>().is_err());
        assert!("|B".parse::<Postfix>().is_err());
        assert!("(A))|(B)?".parse::<Postfix>().is_err());
    }
}
