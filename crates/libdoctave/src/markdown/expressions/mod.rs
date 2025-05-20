use thiserror::Error;

mod filters;
mod interpreter;
mod parser;
mod tokenizer;

pub use interpreter::{Environment, Interpreter, Value};
pub use parser::parse;
pub use tokenizer::is_valid_identifier;

use parser::Operator;

use crate::{render_context::RenderContext, renderable_ast::Position};

use super::error_renderer::{self, Highlight, Location};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Unclosed list in expression")]
    UnclosedList(Span),
    #[error("Unclosed parenthesis in expression")]
    UnclosedParenthesis(Span),
    #[error("Unclosed filter arguments in `{0}`. Expected closing parenthesis")]
    UnclosedFilterArguments(String, Span),
    #[error("Unexpected end of expression")]
    EndOfExpression(Span),
    #[error("Unexpected token `{0}`")]
    UnexpectedToken(String, Span),
    #[error("Unexpected token `{0}`. {1}")]
    UnexpectedTokenWithExpectation(String, String, Span),
    #[error("Unexpected identifier `{0}`")]
    UnexpectedIdentifier(String, Span),
    #[error("Cannot apply operation `{0}` on value `{1}` with type `{2}`")]
    InvalidUnaryOperation(Operator, String, String, Span),
    #[error(
        "Cannot apply operation `{0}` on values `{1}` with type `{2}` and `{3}` with type `{4}`"
    )]
    InvalidBinaryOperation(Operator, String, &'static str, String, &'static str, Span),
    #[error("Unexpected argument to filter `{}`. Expected a `{}` as the {} argument, found `{}` with type `{}`", filter, expected_type, format_arg_index(argument_index), actual_value, actual_type)]
    InvalidArgument {
        filter: &'static str,
        expected_type: &'static str,
        argument_index: usize,
        actual_value: String,
        actual_type: String,
        span: Span,
    },
    #[error("Wrong number of arguments for filter `{0}`. Expected {1} argument(s), found {2}")]
    InvalidArity(&'static str, usize, usize, Span),
    #[error("Expected a filter function on right side of pipeline. Found `{0}`")]
    InvalidFilterPipeline(String, Span),
    #[error("Could not find field `{0}` on `{1}` `{2}`")]
    InvalidDotAccess(String, &'static str, String, Span),
    #[error("Unknown filter `{0}`")]
    UnknownFilter(String, Span),
    #[error("Variable `@{0}` not found")]
    UnknownVariable(String, Span),
    #[error("Unexpected value. Found `{0}`, expected one of {1}")]
    UnexpectedEnum(String, String, Span),
    #[error("Unexpected type. Found `{0}`, expected `{1}`")]
    UnexpectedType(String, String, Span),
    #[error("{0}")]
    FreeformError(String, Span),
}

impl Error {
    pub fn span(&self) -> &Span {
        use Error::*;

        match &self {
            UnclosedList(s) => s,
            UnclosedParenthesis(s) => s,
            UnclosedFilterArguments(_, s) => s,
            EndOfExpression(s) => s,
            UnexpectedToken(_, s) => s,
            UnexpectedTokenWithExpectation(_, _, s) => s,
            UnexpectedIdentifier(_, s) => s,
            InvalidUnaryOperation(_, _, _, s) => s,
            InvalidBinaryOperation(_, _, _, _, _, s) => s,
            InvalidArgument { span: s, .. } => s,
            InvalidArity(_, _, _, s) => s,
            InvalidFilterPipeline(_, s) => s,
            InvalidDotAccess(_, _, _, s) => s,
            UnknownFilter(_, s) => s,
            UnknownVariable(_, s) => s,
            UnexpectedType(_, _, s) => s,
            UnexpectedEnum(_, _, s) => s,
            FreeformError(_, s) => s,
        }
    }

    pub(crate) fn render(
        &self,
        md: &str,
        ctx: &RenderContext,
        key: Option<&str>,
        expr: Option<&str>,
        node_pos: &Position,
    ) -> String {
        let pos = if let (Some(key), Some(expr)) = (key, expr) {
            error_renderer::offset_attribute_error_pos(md, key, expr, node_pos)
        } else {
            node_pos.clone()
        };
        let (location, span) = self.format_expr_error_with_src(md, &pos);

        let highlights = vec![Highlight {
            location,
            msg: None,
            span,
        }];

        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }

    fn format_expr_error_with_src(&self, input: &str, node_pos: &Position) -> (Location, usize) {
        let (err_start_row, err_start_col) =
            self.get_row_col_from_offset(input, node_pos.start.byte_offset + self.span().start + 1);
        let (_, err_end_col) = self.get_row_col_from_offset(
            input,
            node_pos.start.byte_offset + self.span().start + (self.span().end - self.span().start),
        );

        (
            Location::Point(err_start_row, err_start_col),
            // NOTE: Order is important here, because otherwise we'll underflow when the size
            // of the error message is 0.
            1 + err_end_col - err_start_col,
        )
    }

    fn get_row_col_from_offset(&self, input: &str, byte_offset: usize) -> (usize, usize) {
        let mut row = 1;
        let mut col = 1;

        for (index, char) in input.char_indices() {
            if index >= byte_offset {
                break; // Reached or surpassed the target byte offset
            }
            if char == '\n' {
                // Newline character, move to the next row and reset column
                row += 1;
                col = 1;
            } else {
                // Any other character, move to the next column
                col += 1;
            }
        }

        (row, col)
    }
}

fn format_arg_index(index: &usize) -> String {
    match index {
        0 => "first".to_string(),
        1 => "second".to_string(),
        2 => "third".to_string(),
        3 => "fourth".to_string(),
        4 => "fifth".to_string(),
        5 => "sixth".to_string(),
        6 => "seventh".to_string(),
        7 => "eighth".to_string(),
        _ => format!("{}", index + 1),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Span { start, end }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    pub fn evaluate(input: &str, env: Option<interpreter::Environment>) -> Result<Value> {
        let ast = parse(input)?;

        let mut interpreter = Interpreter::new(env);
        interpreter.interpret(ast)
    }

    #[test]
    fn basic_math() {
        let expr = "1 + 1";

        let value = evaluate(expr, None).unwrap();

        assert_eq!(value, Value::Number(2.into()));
    }

    #[test]
    fn grouping() {
        let expr = "(1 + 3) > 1";

        let value = evaluate(expr, None).unwrap();

        assert_eq!(value, Value::Bool(true));
    }

    #[test]
    fn unary() {
        let expr = "-1 + 3";

        let value = evaluate(expr, None).unwrap();

        assert_eq!(value, Value::Number(2.into()));
    }

    #[test]
    fn basic_filters() {
        let expr = "\"foo\" | capitalize | append(\" bar\")";

        let value = evaluate(expr, None).unwrap();

        assert_eq!(value, Value::String("Foo bar".to_string()));
    }

    #[test]
    fn basic_filters_without_pipelines() {
        let expr = "append(capitalize(\"foo\"), \" bar\")";

        let value = evaluate(expr, None).unwrap();

        assert_eq!(value, Value::String("Foo bar".to_string()));
    }

    mod logical_operators {
        use super::*;

        #[test]
        fn or_operator_bools() {
            assert_eq!(evaluate("true || true", None).unwrap(), Value::Bool(true));

            assert_eq!(evaluate("true || false", None).unwrap(), Value::Bool(true));

            assert_eq!(evaluate("false || true", None).unwrap(), Value::Bool(true));

            assert_eq!(
                evaluate("false || false", None).unwrap(),
                Value::Bool(false)
            );
        }

        #[test]
        fn and_operator_bools() {
            assert_eq!(evaluate("true && true", None).unwrap(), Value::Bool(true));

            assert_eq!(evaluate("true && false", None).unwrap(), Value::Bool(false));

            assert_eq!(evaluate("false && true", None).unwrap(), Value::Bool(false));

            assert_eq!(
                evaluate("false && false", None).unwrap(),
                Value::Bool(false)
            );
        }

        #[test]
        fn or_operator_null_shortcircuit() {
            for (left, right, expected) in [
                ("123", "null", Value::Number(123.into())),
                ("null", "123", Value::Number(123.into())),
                ("123", "456", Value::Number(123.into())),
                ("true", "123", Value::Bool(true)),
            ] {
                assert_eq!(
                    evaluate(&format!("{} || {}", left, right), None).unwrap(),
                    expected,
                    "Case `{} || {}` returned {:?}, when expected {:?}",
                    left,
                    right,
                    evaluate(&format!("{} || {}", left, right), None).unwrap(),
                    expected
                );
            }
        }

        #[test]
        fn and_operator_null_shortcircuit() {
            for (left, right, expected) in [
                ("123", "null", Value::Null),
                ("null", "123", Value::Null),
                ("false", "null", Value::Bool(false)),
                ("null", "false", Value::Null),
                ("345", "678", Value::Number(678.into())),
            ] {
                assert_eq!(
                    evaluate(&format!("{} && {}", left, right), None).unwrap(),
                    expected,
                    "Case `{} && {}` returned {:?}, when expected {:?}",
                    left,
                    right,
                    evaluate(&format!("{} && {}", left, right), None).unwrap(),
                    expected
                );
            }
        }
    }

    mod error_handling {
        use super::*;

        #[test]
        fn pipe_into_non_function() {
            let expr = "\"foo\" | 123";

            let error = evaluate(expr, None).unwrap_err();

            assert_eq!(
                error.to_string(),
                "Expected a filter function on right side of pipeline. Found `123`"
            );
        }

        #[test]
        fn unbalanced_pipeline() {
            let expr = "\"foo\" | ";

            let error = evaluate(expr, None).unwrap_err();

            assert_eq!(error.to_string(), "Unexpected end of expression");
        }

        #[test]
        fn invalid_arguments() {
            let expr = "foo(,)";

            let error = evaluate(expr, None).unwrap_err();

            assert_eq!(error.to_string(), "Unexpected token `,`");
        }

        #[test]
        fn unknown_filter() {
            let expr = "123 | lolidontexist";

            let error = evaluate(expr, None).unwrap_err();

            assert_eq!(error.to_string(), "Unknown filter `lolidontexist`");
        }

        #[test]
        fn unknown_function() {
            let expr = "lolidontexist(123)";

            let error = evaluate(expr, None).unwrap_err();

            assert_eq!(error.to_string(), "Unknown filter `lolidontexist`");
        }

        #[test]
        fn invalid_operation() {
            let expr = "2 + true";

            let error = evaluate(expr, None).unwrap_err();

            assert_eq!(
                error.to_string(),
                "Cannot apply operation `+` on values `2` with type `number` and `true` with type `bool`"
            );
        }

        #[test]
        fn invalid_unary_operation() {
            let expr = "-\"foo\"";

            let e = evaluate(expr, None).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Cannot apply operation `-` on value `\"foo\"` with type `string`"
            )
        }

        #[test]
        fn unclosed_function_call() {
            let expr = "foo(1";

            let e = evaluate(expr, None).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Unclosed filter arguments in `foo`. Expected closing parenthesis"
            )
        }

        #[test]
        fn unclosed_grouping() {
            let expr = "1 + (3";

            let e = evaluate(expr, None).unwrap_err();

            assert_eq!(&e.to_string(), "Unclosed parenthesis in expression")
        }

        #[test]
        fn unexpected_token() {
            let expr = "%";

            let e = evaluate(expr, None).unwrap_err();

            assert_eq!(&e.to_string(), "Unexpected token `%`")
        }
    }
}
