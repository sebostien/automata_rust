use super::{Lexer, Lit, ParseError, Token};

/// Tokens in Reverse Polish Notation.
#[derive(Debug, PartialEq, Eq)]
pub struct Postfix {
    pub tokens: Vec<Token>,
}

impl std::str::FromStr for Postfix {
    type Err = ParseError;

    fn from_str(infix: &str) -> Result<Self, Self::Err> {
        let input = &mut Lexer::new(infix);
        let tokens = Self::parse_expr(input, 0)?;
        if let Some(token) = input.next() {
            Err(ParseError::ParsingStopped(token))
        } else {
            Ok(Self { tokens })
        }
    }
}

impl Postfix {
    /// Parse a list of token in postfix notation using [Pratt Parsing].
    ///
    /// [Pratt Parsing]: <https://en.wikipedia.org/wiki/Operator-precedence_parser#Pratt_parsing>
    fn parse_expr(input: &mut Lexer<'_>, prec: usize) -> Result<Vec<Token>, ParseError> {
        let mut lhs = match input.next().ok_or(ParseError::UnexpectedEof)? {
            Token::Lit(lit) => vec![Token::Lit(lit)],
            Token::Eof => vec![Token::Eof],
            Token::OParen => {
                let lhs = Self::parse_expr(input, 0)?;
                if input.next() != Some(Token::CParen) {
                    return Err(ParseError::Unmatched("("));
                }
                lhs
            }
            token => return Err(ParseError::InvalidPrefix(token)),
        };

        while let Some(token) = input.peek() {
            if let Some(post_prec) = token.postfix_precedence() {
                if post_prec < prec {
                    break;
                }
                let token = input.next().unwrap();

                lhs.push(token);
            } else if let Some((left_prec, right_prec)) = token.infix_precedence() {
                if left_prec < prec {
                    break;
                }
                let token = input.next().unwrap();

                let mut rhs = Self::parse_expr(input, right_prec)?;
                if token == Token::Range {
                    let left = lhs.pop().unwrap();
                    let right = rhs.pop().unwrap();
                    if let (Token::Lit(Lit::Char(lower)), Token::Lit(Lit::Char(upper))) =
                        (&left, &right)
                    {
                        lhs.push(Token::Lit(Lit::Range(*lower..=*upper)));
                    } else {
                        return Err(ParseError::InvalidRange {
                            found: format!("({left}-{right})"),
                            expected: "(c-c)",
                        });
                    }
                } else {
                    lhs.append(&mut rhs);
                    lhs.push(token);
                }
            } else {
                break;
            }
        }

        Ok(lhs)
    }
}

impl std::fmt::Display for Postfix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut stack = vec![];

        for token in &self.tokens {
            match token {
                Token::Eof | Token::OParen | Token::CParen | Token::Lit(_) => {
                    stack.push(format!("{token}"));
                }
                Token::Optional | Token::KleeneS | Token::KleeneP => {
                    let lhs = stack.pop().unwrap();
                    stack.push(format!("({lhs}{token})"));
                }
                Token::Range | Token::Concat | Token::Union => {
                    let rhs = stack.pop().unwrap();
                    let lhs = stack.pop().unwrap();
                    stack.push(format!("({lhs}{token}{rhs})"));
                }
            }
        }

        if let Some(s) = stack.pop() {
            write!(f, "{s}")
        } else {
            Ok(())
        }
    }
}
