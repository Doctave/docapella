use std::fmt::Display;

use markdown_rs::mdast;
/// Shared parts of the AST
use serde::Serialize;

use crate::expressions::{self, Interpreter, Result, Value};

/// End and start points in the source file
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Ord, PartialOrd, Eq)]
pub struct Position {
    pub start: Point,
    /// None, if the position is a single point and not a span
    pub end: Point,
}

impl Position {
    pub fn is_point(&self) -> bool {
        self.start == self.end
    }

    pub fn is_span(&self) -> bool {
        self.start != self.end
    }

    pub fn bump_by_byte_offset(&mut self, original_byte_offset: usize, input: &str) {
        self.start.bump_by_byte_offset(original_byte_offset, input);
        self.end.bump_by_byte_offset(original_byte_offset, input);
    }

    pub fn bump_by_byte_and_line_offset(&mut self, line_offset: usize, new_offset: usize) {
        // Bump the byte offset
        self.start.byte_offset += new_offset;
        self.end.byte_offset += new_offset;

        // Bump the row offset
        self.start.row += line_offset;
        self.end.row += line_offset;
    }
}

impl From<&markdown_rs::unist::Position> for Position {
    fn from(value: &markdown_rs::unist::Position) -> Self {
        Position {
            start: Point {
                col: value.start.column,
                row: value.start.line,
                byte_offset: value.start.offset,
            },
            end: Point {
                col: value.end.column,
                row: value.end.line,
                byte_offset: value.end.offset,
            },
        }
    }
}

impl From<&markdown_rs::message::Place> for Position {
    fn from(value: &markdown_rs::message::Place) -> Self {
        match value {
            markdown_rs::message::Place::Point(point) => Position {
                start: Point {
                    col: point.column,
                    row: point.line,
                    byte_offset: point.offset,
                },
                end: Point {
                    col: point.column,
                    row: point.line,
                    byte_offset: point.offset,
                },
            },
            markdown_rs::message::Place::Position(pos) => Position::from(pos),
        }
    }
}

/// A point in a source file
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize, Ord, PartialOrd, Eq)]
pub struct Point {
    /// Column number, 1-indexed
    pub col: usize,
    /// Row number, 1-indexed
    pub row: usize,
    /// Byte offset, 0-indexed
    pub byte_offset: usize,
}

impl Point {
    /// Computes the byte offset from a given row and column and the input string.
    pub(crate) fn byte_offset_from_for_and_col(
        input: &str,
        target_row: usize,
        target_col: usize,
    ) -> usize {
        let mut row = 1;
        let mut col = 1;

        for (byte_offset, char) in input.char_indices() {
            if row == target_row && col == target_col {
                return byte_offset;
            }

            if char == '\n' {
                row += 1;
                col = 1;
            } else {
                col += 1;
            }
        }

        input.len()
    }

    /// Bumps the current point forward by a given byte offset, recomputing the
    /// rows and columns to accomodate.
    ///
    /// The idea is that this Point may refer to some location in a string:
    ///
    /// "foo\n  bar\nbaz"
    ///    ^
    /// But actually, this was part of a larger string:
    ///
    /// "fizzbuzz\n\nfoo\n  bar\nbaz\n  qux"
    ///                ^
    /// So we need to bump the point forward by the length of the string
    /// preceding it, and then recompute the rows and columns.
    ///
    /// The `original_byte_offset` refers to the beginning of the substring
    /// originally used to compute this point (example 1 above).
    ///
    /// `input` is the larger string that this point is actually referring to
    /// and should be used to compute the new rows and columns.
    pub fn bump_by_byte_offset(&mut self, original_byte_offset: usize, input: &str) {
        // Calculate the new byte offset
        self.byte_offset += original_byte_offset;

        // Reset row and col
        self.row = 1;
        self.col = 1;

        for (byte_pos, ch) in input.char_indices() {
            if byte_pos >= self.byte_offset {
                break;
            }

            if ch == '\n' {
                self.row += 1;
                self.col = 1;
            } else {
                self.col += 1;
            }
        }
    }
}

/// An attribute in an MDX-style component
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Attribute {
    pub key: String,
    pub value: Option<AttributeValue>,
}

impl Attribute {
    pub fn evaluate(&self, interpreter: &mut Interpreter) -> Result<Value> {
        match &self.value {
            Some(AttributeValue::Expression(expr)) => {
                let ast = expressions::parse(expr)?;
                interpreter.interpret(ast)
            }
            Some(AttributeValue::Literal(s)) => Ok(Value::String(s.clone())),
            None => Ok(Value::Bool(true)),
        }
    }
}

impl From<mdast::AttributeContent> for Attribute {
    fn from(other: mdast::AttributeContent) -> Self {
        match other {
            mdast::AttributeContent::Property(attr) => Attribute {
                key: attr.name,
                value: attr.value.map(|val| match val {
                    mdast::AttributeValue::Expression(expr) => {
                        AttributeValue::Expression(expr.value)
                    }
                    mdast::AttributeValue::Literal(lit) => AttributeValue::Literal(lit),
                }),
            },
            mdast::AttributeContent::Expression { value, .. } => Attribute {
                key: String::new(),
                value: Some(AttributeValue::Expression(value)),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(untagged, rename_all = "snake_case")]
pub enum AttributeValue {
    Expression(String),
    Literal(String),
}

impl AttributeValue {
    pub fn as_str(&self) -> &str {
        match self {
            AttributeValue::Literal(s) => s.as_str(),
            AttributeValue::Expression(s) => s.as_str(),
        }
    }
}

impl Display for AttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            AttributeValue::Expression(s) => write!(f, "{}", s),
            AttributeValue::Literal(s) => write!(f, "{}", s),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum TableAlignment {
    None,
    Left,
    Right,
    Center,
}

impl From<markdown_rs::mdast::AlignKind> for TableAlignment {
    fn from(value: markdown_rs::mdast::AlignKind) -> Self {
        use markdown_rs::mdast::AlignKind;

        match value {
            AlignKind::Center => Self::Center,
            AlignKind::Left => Self::Left,
            AlignKind::Right => Self::Right,
            AlignKind::None => Self::None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_alignment_serialization() {
        let value = TableAlignment::Center;
        let serialized = serde_json::to_string(&value).unwrap();
        assert_eq!(serialized, r#""center""#);
    }

    #[test]
    fn test_attribute_value_serialization() {
        let value = AttributeValue::Expression("foo".to_string());
        let serialized = serde_json::to_string(&value).unwrap();
        assert_eq!(serialized, r#""foo""#);
    }
}
