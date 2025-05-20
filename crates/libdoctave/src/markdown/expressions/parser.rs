use std::fmt::Display;

use rust_decimal::Decimal;

use super::tokenizer::{Token, Tokenizer};
use super::{Error, Result, Span};

#[derive(Debug, PartialEq, Clone)]
pub struct Expr<'a> {
    pub kind: ExprKind<'a>,
    pub pos: Span,
}

impl<'a> Expr<'a> {
    fn new(kind: ExprKind<'a>, pos: Span) -> Self {
        Expr { kind, pos }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum ExprKind<'a> {
    Filter(&'a str, Vec<Expr<'a>>),
    Unary(Operator, Box<Expr<'a>>),
    Binary(Operator, Box<Expr<'a>>, Box<Expr<'a>>),
    List(Vec<Expr<'a>>),
    Grouping(Box<Expr<'a>>),
    Number(Decimal),
    String(&'a str),
    Identifier(&'a str),
    VariableAccess(&'a str),
    DotAccess(Box<Expr<'a>>, &'a str),
    Bool(bool),
    Null,
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operator {
    Plus,
    Minus,
    Slash,
    Star,
    GreaterThan,
    GreaterOrEqualThan,
    LessThan,
    LessOrEqualThan,
    EqualsEquals,
    NotEqual,
    LogicalAnd,
    LogicalOr,
}

impl Display for Operator {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Operator::*;

        match self {
            Plus => f.write_str("+"),
            Minus => f.write_str("-"),
            Slash => f.write_str("/"),
            Star => f.write_str("*"),
            GreaterThan => f.write_str(">"),
            GreaterOrEqualThan => f.write_str(">="),
            LessThan => f.write_str("<"),
            LessOrEqualThan => f.write_str("<="),
            EqualsEquals => f.write_str("=="),
            NotEqual => f.write_str("!="),
            LogicalAnd => f.write_str("&&"),
            LogicalOr => f.write_str("||"),
        }
    }
}

pub fn parse(input: &str) -> Result<Expr> {
    let mut parser = Parser::new(input);

    parser.parse()
}

#[derive(Debug)]
pub struct Parser<'a> {
    tokenizer: std::iter::Peekable<Tokenizer<'a>>,
    offset: usize,
    input: &'a str,
}

impl<'a> Parser<'a> {
    fn new(input: &'a str) -> Self {
        let tokenizer = Tokenizer::new(input).peekable();

        Self {
            tokenizer,
            input,
            offset: 0,
        }
    }

    #[allow(dead_code)]
    pub fn offset(&self) -> usize {
        self.offset
    }

    fn next(&mut self) -> Option<(Token<'a>, usize, usize)> {
        match self.tokenizer.next() {
            Some((t, start, end)) => {
                self.offset = end;
                Some((t, start, end))
            }
            None => None,
        }
    }

    fn peek(&mut self) -> Option<&(Token, usize, usize)> {
        self.tokenizer.peek()
    }

    fn parse(&mut self) -> Result<Expr<'a>> {
        self.parse_expression()
    }

    fn parse_expression(&mut self) -> Result<Expr<'a>> {
        self.parse_filter_chain()
    }

    fn parse_filter_chain(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_comparison()?;

        while matches!(self.peek().map(|(t, _, _)| t), Some(Token::Pipe)) {
            // Consume pipe
            self.next();
            let start_pos = expr.pos.clone();

            match self.next() {
                Some((Token::Identifier(filter_name), _, _)) => {
                    let mut args = Vec::new();
                    args.push(expr);

                    // Check if there are arguments to the filter
                    if matches!(self.peek(), Some((Token::OpenParen, _, _))) {
                        self.next(); // Consume the '(' token
                                     // Parse arguments until you reach ')'
                        while !matches!(self.peek(), Some((Token::CloseParen, _, _))) {
                            args.push(self.parse_expression()?);
                            if matches!(self.peek(), Some((Token::Comma, _, _))) {
                                self.next(); // Consume the ',' token
                            }
                        }
                        self.next(); // Consume the ')' token
                    }

                    // Create a new Expr::Filter node with the filter_name and arguments
                    expr = Expr {
                        pos: Span::new(start_pos.start, self.offset), // Update with correct end
                        kind: ExprKind::Filter(filter_name, args),
                    };
                }
                Some((_, start, end)) => {
                    return Err(Error::InvalidFilterPipeline(
                        self.input[start..end].to_owned(),
                        Span::new(start, end),
                    ))
                }
                None => {
                    return Err(Error::EndOfExpression(Span::new(
                        expr.pos.start,
                        self.offset,
                    )))
                }
            }
        }

        Ok(expr)
    }

    fn parse_comparison(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_add_subtract()?;

        while matches!(
            self.peek().map(|(t, _, _)| t),
            Some(Token::GreaterThan)
                | Some(Token::GreaterOrEqualThan)
                | Some(Token::LessThan)
                | Some(Token::LessOrEqualThan)
                | Some(Token::EqualsEquals)
                | Some(Token::NotEqual)
                | Some(Token::And)
                | Some(Token::Or)
        ) {
            let op = match self.next().unwrap() {
                (Token::GreaterThan, _, _) => Operator::GreaterThan,
                (Token::GreaterOrEqualThan, _, _) => Operator::GreaterOrEqualThan,
                (Token::LessThan, _, _) => Operator::LessThan,
                (Token::LessOrEqualThan, _, _) => Operator::LessOrEqualThan,
                (Token::EqualsEquals, _, _) => Operator::EqualsEquals,
                (Token::NotEqual, _, _) => Operator::NotEqual,
                (Token::And, _, _) => Operator::LogicalAnd,
                (Token::Or, _, _) => Operator::LogicalOr,
                _ => unreachable!(),
            };
            let right = self.parse_add_subtract()?;

            expr = Expr {
                pos: Span::new(expr.pos.start, right.pos.end),
                kind: ExprKind::Binary(op, Box::new(expr), Box::new(right)),
            };
        }

        Ok(expr)
    }

    fn parse_add_subtract(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_factor()?;

        while matches!(
            self.peek().map(|(t, _, _)| t),
            Some(Token::Plus) | Some(Token::Minus)
        ) {
            let op = match self.next().unwrap() {
                (Token::Plus, _, _) => Operator::Plus,
                (Token::Minus, _, _) => Operator::Minus,
                _ => unreachable!(),
            };
            let right = self.parse_factor()?;

            expr = Expr {
                pos: Span::new(expr.pos.start, right.pos.end),
                kind: ExprKind::Binary(op, Box::new(expr), Box::new(right)),
            };
        }

        Ok(expr)
    }

    fn parse_factor(&mut self) -> Result<Expr<'a>> {
        let mut expr = self.parse_unary()?;

        while matches!(
            self.peek().map(|(t, _, _)| t),
            Some(Token::Star) | Some(Token::Slash)
        ) {
            let op = match self.next().unwrap() {
                (Token::Slash, _, _) => Operator::Slash,
                (Token::Star, _, _) => Operator::Star,
                _ => unreachable!(),
            };
            let right = self.parse_unary()?;

            expr = Expr {
                pos: Span::new(expr.pos.start, right.pos.end),
                kind: ExprKind::Binary(op, Box::new(expr), Box::new(right)),
            };
        }

        Ok(expr)
    }

    fn parse_unary(&mut self) -> Result<Expr<'a>> {
        if matches!(self.peek(), Some((Token::Minus, _, _))) {
            let (_op, start, _) = self.next().unwrap();
            let expr = self.parse_unary()?;

            Ok(Expr {
                pos: Span::new(start, expr.pos.end),
                kind: ExprKind::Unary(Operator::Minus, Box::new(expr)),
            })
        } else {
            self.parse_terminal()
        }
    }

    fn parse_terminal(&mut self) -> Result<Expr<'a>> {
        let r = match self.next() {
            Some((Token::OpenSquareBacket, list_start, _)) => {
                let mut exprs = vec![];
                let list_end;

                loop {
                    let last = self.peek();

                    match last {
                        Some((Token::CloseSquareBacket, _start, end)) => {
                            list_end = *end;
                            self.next().unwrap();
                            break;
                        }
                        Some(_) => {
                            exprs.push(self.parse_expression()?);

                            if matches!(self.peek(), Some((Token::Comma, _, _))) {
                                self.next().unwrap();
                            }
                        }
                        None => Err(Error::UnclosedList(Span::new(list_start, self.input.len())))?,
                    };
                }

                Ok(Expr {
                    kind: ExprKind::List(exprs),
                    pos: Span::new(list_start, list_end),
                })
            }
            Some((Token::OpenParen, grouping_start, _)) => {
                let expr = self.parse_expression()?;

                // Check the expression closes
                match self.next() {
                    Some((Token::CloseParen, _, grouping_end)) => Ok(Expr {
                        kind: ExprKind::Grouping(Box::new(expr)),
                        pos: Span::new(grouping_start, grouping_end),
                    }),
                    Some((_t, _start, end)) => {
                        Err(Error::UnclosedParenthesis(Span::new(grouping_start, end)))
                    }
                    None => Err(Error::UnclosedParenthesis(Span::new(
                        grouping_start,
                        self.input.len(),
                    ))),
                }
            }
            Some((Token::True, s, e)) => Ok(Expr::new(ExprKind::Bool(true), Span::new(s, e))),
            Some((Token::False, s, e)) => Ok(Expr::new(ExprKind::Bool(false), Span::new(s, e))),
            Some((Token::Null, s, e)) => Ok(Expr::new(ExprKind::Null, Span::new(s, e))),
            Some((Token::Number(i), s, e)) => Ok(Expr::new(ExprKind::Number(i), Span::new(s, e))),
            Some((Token::String(t), s, e)) => Ok(Expr::new(ExprKind::String(t), Span::new(s, e))),
            Some((Token::At, s, _)) => match self.next() {
                Some((Token::Identifier(ident), _, ident_e)) => {
                    let base_expr = Expr {
                        kind: ExprKind::VariableAccess(ident),
                        pos: Span::new(s, ident_e),
                    };

                    self.parse_dot_access(base_expr)
                }
                Some((_, other_s, other_e)) => Err(Error::UnexpectedTokenWithExpectation(
                    self.input[other_s..other_e].to_owned(),
                    "Expected identifier after `@` symbol".to_string(),
                    Span::new(self.offset, self.input.len()),
                )),
                None => Err(Error::EndOfExpression(Span::new(
                    self.offset,
                    self.input.len(),
                ))),
            },
            Some((Token::Identifier(i), s, e)) => {
                if matches!(self.peek(), Some((Token::OpenParen, _, _))) {
                    self.parse_function_call(i, s)
                } else {
                    Ok(Expr::new(ExprKind::Identifier(i), Span::new(s, e)))
                }
            }
            Some((_unexpected_token, s, e)) => Err(Error::UnexpectedToken(
                self.input[s..e].to_owned(),
                Span::new(s, e),
            )),
            None => Err(Error::EndOfExpression(Span::new(
                self.offset,
                self.input.len(),
            ))),
        }?;

        Ok(r)
    }

    fn parse_function_call(&mut self, identifier: &'a str, start: usize) -> Result<Expr<'a>> {
        let mut args = Vec::new();
        // Consume open paren paren
        self.next();

        if !matches!(self.peek(), Some((Token::CloseParen, _, _))) {
            loop {
                args.push(self.parse_expression()?);
                if !matches!(self.peek(), Some((Token::Comma, _, _))) {
                    break;
                }
                self.next(); // Consume the comma
            }
        }
        let end_pos = match self.next() {
            // Consume the closing parenthesis
            Some((Token::CloseParen, _, end)) => end,
            _ => Err(Error::UnclosedFilterArguments(
                identifier.to_owned(),
                Span::new(start, self.offset),
            ))?,
        };

        Ok(Expr {
            pos: Span::new(start, end_pos),
            kind: ExprKind::Filter(identifier, args),
        })
    }

    fn parse_dot_access(&mut self, mut expr: Expr<'a>) -> Result<Expr<'a>> {
        while matches!(self.peek(), Some((Token::Dot, _, _))) {
            let t = self.next().unwrap(); // Consume the dot
            match self.next() {
                Some((Token::Identifier(property), _, e)) => {
                    expr = Expr {
                        pos: Span::new(t.1, e),
                        kind: ExprKind::DotAccess(Box::new(expr), property),
                    };
                }
                Some((_unexpected_token, s, e)) => {
                    return Err(Error::UnexpectedTokenWithExpectation(
                        self.input[s..e].to_owned(),
                        "Expected identifier after `.`".to_string(),
                        Span::new(s, e),
                    ))
                }
                None => {
                    return Err(Error::EndOfExpression(Span::new(
                        self.offset,
                        self.input.len(),
                    )))
                }
            }
        }
        Ok(expr)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let input = "1 + 2";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::Binary(
                    Operator::Plus,
                    Box::new(Expr {
                        kind: ExprKind::Number(1.into()),
                        pos: Span::new(0, 1)
                    }),
                    Box::new(Expr {
                        kind: ExprKind::Number(2.into()),
                        pos: Span::new(4, 5)
                    })
                ),
                pos: Span::new(0, 5)
            }
        );
    }

    #[test]
    fn filter_chain() {
        // Basically the same as
        //
        //   append(capitalize("foo"), " bar")
        //
        let input = "\"foo\" | capitalize | append(\" bar\")";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::Filter(
                    "append",
                    vec![
                        Expr {
                            kind: ExprKind::Filter(
                                "capitalize",
                                vec![Expr {
                                    kind: ExprKind::String("foo"),
                                    pos: Span::new(0, 5),
                                }]
                            ),
                            pos: Span::new(0, 18),
                        },
                        Expr {
                            kind: ExprKind::String(" bar"),
                            pos: Span::new(28, 34),
                        }
                    ],
                ),
                pos: Span::new(0, 35)
            }
        );
    }

    #[test]
    fn filter_chain_equivalence() {
        let input = "append(capitalize(\"foo\"), \" bar\")";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::Filter(
                    "append",
                    vec![
                        Expr {
                            kind: ExprKind::Filter(
                                "capitalize",
                                vec![Expr {
                                    kind: ExprKind::String("foo"),
                                    pos: Span::new(18, 23),
                                }]
                            ),
                            pos: Span::new(7, 24),
                        },
                        Expr {
                            kind: ExprKind::String(" bar"),
                            pos: Span::new(26, 32),
                        }
                    ],
                ),
                pos: Span::new(0, 33)
            }
        );
    }

    #[test]
    fn function_without_args() {
        let input = "example()";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::Filter("example", vec![]),
                pos: Span::new(0, 9)
            }
        );
    }

    #[test]
    fn variable_access() {
        let input = "@foo";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::VariableAccess("foo"),
                pos: Span::new(0, 4)
            }
        );
    }

    #[test]
    fn variable_access_nested() {
        let input = "@foo.bar";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::DotAccess(
                    Box::new(Expr {
                        kind: ExprKind::VariableAccess("foo"),
                        pos: Span::new(0, 4)
                    }),
                    "bar"
                ),
                pos: Span::new(4, 8)
            },
        );
    }

    #[test]
    fn variable_access_arbitrarily_nested() {
        let input = "@foo.bar.baz";

        let ast = parse(input).unwrap();

        assert_eq!(
            ast,
            Expr {
                kind: ExprKind::DotAccess(
                    Box::new(Expr {
                        kind: ExprKind::DotAccess(
                            Box::new(Expr {
                                kind: ExprKind::VariableAccess("foo"),
                                pos: Span::new(0, 4)
                            }),
                            "bar"
                        ),
                        pos: Span::new(4, 8)
                    }),
                    "baz"
                ),
                pos: Span::new(8, 12)
            }
        );
    }
}
