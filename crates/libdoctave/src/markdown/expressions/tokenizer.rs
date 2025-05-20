use std::str::FromStr;

use rust_decimal::Decimal;

#[derive(Debug, PartialEq)]
pub(super) enum Token<'a> {
    True,
    False,
    Plus,
    Minus,
    Slash,
    Star,
    Equals,
    EqualsEquals,
    Or,
    And,
    NotEqual,
    LessThan,
    LessOrEqualThan,
    GreaterThan,
    GreaterOrEqualThan,
    OpenParen,
    CloseParen,
    OpenSquareBacket,
    CloseSquareBacket,
    OpenCurlyBrace,
    CloseCurlyBrace,
    Dot,
    Comma,
    Pipe,
    At,
    Null,
    Number(Decimal),
    Identifier(&'a str),
    String(&'a str),
    /// A fallback when we were unable to parse the token into anything meaningful. It didn't match
    /// identifier rules, and didn't match "true"/"false", etc.
    Unknown(&'a str),
}

pub fn is_valid_identifier(input: &str) -> bool {
    !input.is_empty()
        && input
            .chars()
            .next()
            .map(|c| c.is_ascii_alphabetic())
            .unwrap_or(false)
        && input.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[derive(Debug)]
pub(super) struct Tokenizer<'a> {
    input: &'a str,
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    cursor: usize,
    current_token_start: usize,
    current_token_end: usize,
}

impl<'a> Tokenizer<'a> {
    pub fn new(input: &'a str) -> Self {
        let chars = input.chars().peekable();

        Tokenizer {
            input,
            chars,
            cursor: 0,
            current_token_start: 0,
            current_token_end: 0,
        }
    }

    fn consume_number(&mut self) -> &'a str {
        let mut found_period = false;
        while let Some(c) = self.peek_char() {
            if c.is_ascii_digit() {
                self.consume_char();
            } else if c == &'.' && !found_period {
                self.consume_char();
                found_period = true;
            } else {
                break;
            }
        }

        &self.input[self.current_token_start..self.cursor]
    }

    fn consume_word(&mut self) -> &'a str {
        while let Some(c) = self.peek_char() {
            if c.is_ascii_alphanumeric() || *c == '_' {
                self.consume_char();
                continue;
            } else {
                break;
            }
        }

        &self.input[self.current_token_start..self.cursor]
    }

    fn consume_quoted_text(&mut self) -> Token<'a> {
        let mut found_closing_tag = false;

        // Consume starting quote
        self.consume_char();

        while let Some(c) = self.peek_char() {
            if *c == '"' {
                self.consume_char();
                found_closing_tag = true;

                break;
            } else if self.consume_char().is_none() {
                break;
            }
        }

        if found_closing_tag {
            Token::String(&self.input[(self.current_token_start + 1)..self.cursor - 1])
        } else {
            Token::Unknown(&self.input[(self.current_token_start)..self.cursor])
        }
    }

    fn consume_char(&mut self) -> Option<char> {
        if self.cursor >= self.input.len() {
            None
        } else {
            self.cursor += 1;
            self.chars.next()
        }
    }

    fn peek_char(&mut self) -> Option<&char> {
        self.chars.peek()
    }
}

impl<'a> Iterator for Tokenizer<'a> {
    type Item = (Token<'a>, usize, usize);

    fn next(&mut self) -> Option<Self::Item> {
        if self.cursor >= self.input.len() {
            return None;
        }

        self.current_token_start = self.cursor;

        let token = loop {
            match self.consume_char() {
                Some('=') => {
                    if let Some('=') = self.peek_char() {
                        self.consume_char();
                        break Token::EqualsEquals;
                    } else {
                        break Token::Equals;
                    }
                }
                Some('<') => {
                    if let Some('=') = self.peek_char() {
                        self.consume_char();
                        break Token::LessOrEqualThan;
                    } else {
                        break Token::LessThan;
                    }
                }
                Some('>') => {
                    if let Some('=') = self.peek_char() {
                        self.consume_char();
                        break Token::GreaterOrEqualThan;
                    } else {
                        break Token::GreaterThan;
                    }
                }
                Some('|') => {
                    if let Some('|') = self.peek_char() {
                        self.consume_char();
                        break Token::Or;
                    }

                    break Token::Pipe;
                }
                Some('@') => break Token::At,
                Some('.') => break Token::Dot,
                Some(',') => break Token::Comma,
                Some('+') => break Token::Plus,
                Some('-') => break Token::Minus,
                Some('/') => break Token::Slash,
                Some('*') => break Token::Star,
                Some('(') => break Token::OpenParen,
                Some(')') => break Token::CloseParen,
                Some('{') => break Token::OpenCurlyBrace,
                Some('}') => break Token::CloseCurlyBrace,
                Some('[') => break Token::OpenSquareBacket,
                Some(']') => break Token::CloseSquareBacket,
                Some('"') => break self.consume_quoted_text(),
                Some(c) => {
                    if c.is_whitespace() {
                        self.current_token_start = self.cursor;
                        continue;
                    } else if c == '!' && self.peek_char() == Some(&'=') {
                        self.consume_char();
                        break Token::NotEqual;
                    } else if c.is_ascii_digit() {
                        let str_num = self.consume_number();

                        break Token::Number(
                            Decimal::from_str(str_num)
                                .expect("Tokenizer guarantees we only get valid values here"),
                        );
                    } else {
                        let text = self.consume_word();

                        match text {
                            "true" => break Token::True,
                            "false" => break Token::False,
                            "null" => break Token::Null,
                            "&" => {
                                if let Some('&') = self.peek_char() {
                                    self.consume_char();
                                    break Token::And;
                                }

                                break Token::Unknown(text);
                            }
                            _ => {
                                if is_valid_identifier(text) {
                                    break Token::Identifier(text);
                                } else {
                                    break Token::Unknown(text);
                                }
                            }
                        }
                    }
                }

                None => return None,
            }
        };

        self.current_token_end = self.cursor;

        Some((token, self.current_token_start, self.current_token_end))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic() {
        let input =
            "12 34.5 \"fizz buzz\" foo + / - * { } | [ ] ( ) = == < > >= <= bar(a,b) @balloon.rope null && ||";

        let tokens = Tokenizer::new(input).map(|(t, _, _)| t).collect::<Vec<_>>();

        assert_eq!(
            tokens,
            vec![
                Token::Number(12.into()),
                Token::Number(Decimal::from_str("34.5").unwrap()),
                Token::String("fizz buzz"),
                Token::Identifier("foo"),
                Token::Plus,
                Token::Slash,
                Token::Minus,
                Token::Star,
                Token::OpenCurlyBrace,
                Token::CloseCurlyBrace,
                Token::Pipe,
                Token::OpenSquareBacket,
                Token::CloseSquareBacket,
                Token::OpenParen,
                Token::CloseParen,
                Token::Equals,
                Token::EqualsEquals,
                Token::LessThan,
                Token::GreaterThan,
                Token::GreaterOrEqualThan,
                Token::LessOrEqualThan,
                Token::Identifier("bar"),
                Token::OpenParen,
                Token::Identifier("a"),
                Token::Comma,
                Token::Identifier("b"),
                Token::CloseParen,
                Token::At,
                Token::Identifier("balloon"),
                Token::Dot,
                Token::Identifier("rope"),
                Token::Null,
                Token::And,
                Token::Or,
            ]
        );
    }

    #[test]
    fn string_starting_pos() {
        let input = "\"foo\"";

        let tokens = Tokenizer::new(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![(Token::String("foo"), 0, 5)]);
    }

    #[test]
    fn preceding_whitespace() {
        let input = "  \"foo\"";

        let tokens = Tokenizer::new(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![(Token::String("foo"), 2, 7)]);
    }

    #[test]
    fn handles_unclosed_string_literal() {
        let input = "  \"foo";

        let tokens = Tokenizer::new(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![(Token::Unknown("\"foo"), 2, 6)]);
    }

    #[test]
    fn unknown_token() {
        let input = "%";

        let tokens = Tokenizer::new(input).collect::<Vec<_>>();

        assert_eq!(tokens, vec![(Token::Unknown("%"), 0, 1)]);
    }
}
