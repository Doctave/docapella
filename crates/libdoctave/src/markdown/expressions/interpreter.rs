use std::collections::HashMap;
use std::fmt::Display;

use indexmap::IndexMap;
use rust_decimal::Decimal;

use crate::markdown::custom_components::attribute::AttributeTypeValue;

use super::parser::{Expr, ExprKind, Operator};
use super::{filters, Error, Result, Span};

#[derive(Debug)]
pub struct Environment {
    scopes: Vec<Scope>,
}

impl Environment {
    pub fn from_user_prefs(prefs: &HashMap<String, String>) -> Self {
        let mut env_prefs = IndexMap::new();

        for (key, value) in prefs {
            env_prefs.insert(key.to_owned(), Value::String(value.to_owned()));
        }

        let mut env = Environment::default();
        env.add_global("user_preferences", Value::Object(env_prefs));
        env
    }

    pub fn add_global<I: Into<String>>(&mut self, identifier: I, val: Value) {
        if let Some(s) = self.scopes.first_mut() {
            s.add(identifier.into(), val)
        }
    }

    fn lookup(&self, identifier: &str) -> Option<&Value> {
        if let Some(s) = self.scopes.first() {
            s.lookup(identifier)
        } else {
            None
        }
    }
}

impl Default for Environment {
    fn default() -> Self {
        Environment {
            scopes: vec![Scope::default()],
        }
    }
}

#[derive(Debug, Default)]
struct Scope {
    data: HashMap<String, Value>,
}

impl Scope {
    #[allow(dead_code)]
    fn add<I: Into<String>>(&mut self, ident: I, val: Value) {
        self.data.insert(ident.into(), val);
    }

    fn lookup(&self, identifier: &str) -> Option<&Value> {
        self.data.get(identifier)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Number(Decimal),
    String(String),
    Bool(bool),
    List(Vec<Value>),
    Object(IndexMap<String, Value>),
    Null,
}

impl Value {
    pub fn debug_string(&self) -> String {
        match self {
            Value::Number(v) => format!("{:?}", v),
            Value::String(v) => format!("{:?}", v),
            Value::Bool(v) => format!("{:?}", v),
            Value::List(v) => format!("{:?}", v),
            Value::Object(v) => {
                let mut out = String::new();
                out.push_str("{ ");

                for (i, (key, value)) in v.iter().enumerate() {
                    out.push_str(key);
                    out.push_str(": ");
                    out.push_str(&value.debug_string());

                    if i < v.len() - 1 {
                        out.push_str(", ");
                    }
                }

                out.push_str(" }");

                out
            }
            Value::Null => "null".to_string(),
        }
    }

    pub fn type_name(&self) -> &'static str {
        match self {
            Value::Number(_) => "number",
            Value::String(_) => "string",
            Value::Bool(_) => "bool",
            Value::List(_) => "list",
            Value::Object(_) => "object",
            Value::Null => "null",
        }
    }

    pub fn is_truthy(&self) -> bool {
        match self {
            Value::Number(_) => true,
            Value::String(_) => true,
            Value::Bool(b) => *b,
            Value::List(_) => true,
            Value::Object(_) => true,
            Value::Null => false,
        }
    }
}

impl From<AttributeTypeValue> for Value {
    fn from(value: AttributeTypeValue) -> Self {
        match value {
            AttributeTypeValue::Number(i) => Value::Number(i.into()),
            AttributeTypeValue::Text(s) => Value::String(s),
            AttributeTypeValue::Boolean(b) => Value::Bool(b),
        }
    }
}

impl Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Number(i) => write!(f, "{}", i),
            Value::Bool(b) => write!(f, "{}", b),
            Value::String(t) => write!(f, "{}", t),
            Value::List(list) => {
                write!(f, "[")?;
                for (i, e) in list.iter().enumerate() {
                    write!(f, "{}", e)?;
                    if i < list.len() - 1 {
                        write!(f, ",")?;
                    }
                }
                write!(f, "]")
            }
            Value::Object(_) => {
                write!(f, "{}", self.debug_string())
            }
            Value::Null => write!(f, ""),
        }
    }
}

pub struct Interpreter {
    pub env: Environment,
}

impl Interpreter {
    pub fn new(env: Option<Environment>) -> Self {
        Interpreter {
            env: env.unwrap_or_default(),
        }
    }

    pub fn interpret(&mut self, expr: Expr) -> Result<Value> {
        match expr.kind {
            ExprKind::Grouping(g) => self.interpret(*g),
            ExprKind::Number(i) => Ok(Value::Number(i)),
            ExprKind::Filter(name, args) => {
                let mut arg_values = vec![];
                for arg in args {
                    arg_values.push(self.interpret(arg)?);
                }

                filters::apply_filter(name, arg_values, expr.pos)
            }
            ExprKind::String(s) => Ok(Value::String(s.to_owned())),
            ExprKind::Bool(b) => Ok(Value::Bool(b)),
            ExprKind::Null => Ok(Value::Null),
            ExprKind::Binary(op, left, right) => {
                let left = self.interpret(*left)?;
                let right = self.interpret(*right)?;
                self.evaluate_binary_op(op, left, right, expr.pos)
            }
            ExprKind::List(l) => {
                let mut out = vec![];
                for expr in l {
                    out.push(self.interpret(expr)?);
                }

                Ok(Value::List(out))
            }
            ExprKind::Unary(op, expr) => {
                // Store offset because `interpret` moves the expr
                let start = expr.pos.start;
                let end = expr.pos.end;

                match self.interpret(*expr)? {
                    Value::Number(i) => Ok(Value::Number(-i)),
                    other => Err(Error::InvalidUnaryOperation(
                        op,
                        other.debug_string(),
                        other.type_name().to_owned(),
                        Span::new(start, end),
                    )),
                }
            }
            ExprKind::VariableAccess(identifier) => match self.env.lookup(identifier) {
                Some(v) => Ok(v.clone()),
                None => Err(Error::UnknownVariable(
                    identifier.to_owned(),
                    Span::new(expr.pos.start, expr.pos.end),
                )),
            },
            ExprKind::DotAccess(expr, identifier) => {
                // Store offset because `interpret` moves the expr
                let start = expr.pos.start;
                let end = expr.pos.end;

                match self.interpret(*expr)? {
                    ref obj @ Value::Object(ref v) => Ok(v
                        .get(identifier)
                        .ok_or(Error::InvalidDotAccess(
                            identifier.to_owned(),
                            obj.type_name(),
                            obj.debug_string(),
                            Span::new(start, end),
                        ))?
                        .clone()),
                    other => Err(Error::InvalidDotAccess(
                        identifier.to_owned(),
                        other.type_name(),
                        other.debug_string(),
                        Span::new(start, end),
                    )),
                }
            }
            ExprKind::Identifier(identifier) => Err(Error::UnexpectedIdentifier(
                identifier.to_owned(),
                Span::new(expr.pos.start, expr.pos.end),
            )),
        }
    }

    fn evaluate_binary_op(
        &mut self,
        op: Operator,
        left: Value,
        right: Value,
        pos: Span,
    ) -> Result<Value> {
        use Operator::*;
        use Value::*;

        match (&op, &left, &right) {
            (Plus, Number(x), Number(y)) => Ok(Number(x + y)),
            (Minus, Number(x), Number(y)) => Ok(Number(x - y)),
            (Star, Number(x), Number(y)) => Ok(Number(x * y)),
            (Slash, Number(x), Number(y)) => Ok(Number(x / y)),
            (GreaterThan, Number(x), Number(y)) => Ok(Bool(x > y)),
            (GreaterOrEqualThan, Number(x), Number(y)) => Ok(Bool(x >= y)),
            (LessThan, Number(x), Number(y)) => Ok(Bool(x < y)),
            (LessOrEqualThan, Number(x), Number(y)) => Ok(Bool(x <= y)),
            (EqualsEquals, Number(x), Number(y)) => Ok(Bool(x == y)),
            (EqualsEquals, Bool(x), Bool(y)) => Ok(Bool(x == y)),
            (EqualsEquals, String(x), String(y)) => Ok(Bool(x == y)),
            (EqualsEquals, Null, Null) => Ok(Bool(true)),
            (EqualsEquals, _, Null) => Ok(Bool(false)),
            (EqualsEquals, Null, _) => Ok(Bool(false)),
            (NotEqual, Number(x), Number(y)) => Ok(Bool(x != y)),
            (NotEqual, Bool(x), Bool(y)) => Ok(Bool(x != y)),
            (NotEqual, String(x), String(y)) => Ok(Bool(x != y)),
            (NotEqual, Null, Null) => Ok(Bool(false)),
            (NotEqual, _, Null) => Ok(Bool(true)),
            (NotEqual, Null, _) => Ok(Bool(true)),
            (LogicalOr, left, right) => {
                if left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            (LogicalAnd, left, right) => {
                if !left.is_truthy() {
                    Ok(left.clone())
                } else {
                    Ok(right.clone())
                }
            }
            _ => Err(Error::InvalidBinaryOperation(
                op,
                left.debug_string(),
                left.type_name(),
                right.debug_string(),
                right.type_name(),
                pos,
            )),
        }
    }
}

#[cfg(test)]
mod test {
    use std::str::FromStr;

    use super::*;

    fn evaluate(expr: &str) -> super::super::Result<Value> {
        super::super::test::evaluate(expr, None)
    }

    fn evaluate_with_env(expr: &str, env: Environment) -> super::super::Result<Value> {
        super::super::test::evaluate(expr, Some(env))
    }

    #[test]
    fn plus_int() {
        let expr = "1 + 3";

        assert_eq!(evaluate(expr).unwrap(), Value::Number(4.into()));
    }

    #[test]
    fn plus_float() {
        let expr = "1.4 + 1.4";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("2.8").unwrap())
        );
    }

    #[test]
    fn plus_int_float() {
        let expr = "1 + 1.4";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("2.4").unwrap())
        );

        let expr = "1.4 + 2";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("3.4").unwrap())
        );
    }

    #[test]
    fn minus_int() {
        let expr = "1 - 3";

        assert_eq!(evaluate(expr).unwrap(), Value::Number((-2).into()));
    }

    #[test]
    fn minus_float() {
        let expr = "1.4 - 3.4";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("-2.0").unwrap())
        );
    }

    #[test]
    fn minus_int_float() {
        let expr = "1 - 0.4";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("0.6").unwrap())
        );

        let expr = "2.5 - 1";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("1.5").unwrap())
        );
    }

    #[test]
    fn multiply_int() {
        let expr = "1 * 4";

        assert_eq!(evaluate(expr).unwrap(), Value::Number(4.into()));
    }

    #[test]
    fn multiply_float() {
        let expr = "2.0 * 1.5";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("3.0").unwrap())
        );
    }

    #[test]
    fn multiply_int_float() {
        let expr = "2 * 1.5";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("3.0").unwrap())
        );

        let expr = "1.5 * 2";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("3.0").unwrap())
        );
    }

    #[test]
    fn divide_int() {
        let expr = "3 / 2";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("1.5").unwrap())
        );
    }

    #[test]
    fn divide_float() {
        let expr = "4.0 / 2.0";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("2.0").unwrap())
        );
    }

    #[test]
    fn divide_int_float() {
        let expr = "3 / 2.0";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("1.5").unwrap())
        );

        let expr = "3.0 / 2";

        assert_eq!(
            evaluate(expr).unwrap(),
            Value::Number(Decimal::from_str("1.5").unwrap())
        );
    }

    #[test]
    fn greater_than_int() {
        let expr = "4 > 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn less_than_int() {
        let expr = "4 < 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn greater_than_int_float() {
        let expr = "4.0 > 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4 > 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn less_than_int_float() {
        let expr = "4.0 < 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4 < 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn greater_or_equal_than_int() {
        let expr = "4 >= 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn less_or_equal_than_int() {
        let expr = "4 <= 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn greater_than_float() {
        let expr = "4.0 > 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn less_than_float() {
        let expr = "4.0 < 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn greater_or_equal_than_float() {
        let expr = "4.0 >= 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn less_or_equal_than_float() {
        let expr = "4.0 <= 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn greater_or_equal_than_int_float() {
        let expr = "4.0 >= 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4 >= 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn less_or_equal_than_int_float() {
        let expr = "4.0 <= 2";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4 <= 2.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn equals_equals_int() {
        let expr = "4 == 4";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4 == 3";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn equals_equals_float() {
        let expr = "4.0 == 4.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4.0 == 3.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn equals_equals_bool() {
        let expr = "true == true";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "false == true";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn equals_equals_float_int() {
        let expr = "4.0 == 4";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4.0 == 3";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4 == 4.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4 == 3.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn equals_equals_string() {
        let expr = "\"foo\" == \"foo\"";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "\"foo\" == \"bar\"";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));
    }

    #[test]
    fn not_equal_int() {
        let expr = "4 != 4";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4 != 3";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn not_equal_float() {
        let expr = "4.0 != 4.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4.0 != 3.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn not_equal_bool() {
        let expr = "true != true";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "false != true";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn not_equal_float_int() {
        let expr = "4.0 != 4";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4.0 != 3";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));

        let expr = "4 != 4.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "4 != 3.0";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn not_equal_string() {
        let expr = "\"foo\" != \"foo\"";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(false));

        let expr = "\"foo\" != \"bar\"";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn null_equals_null() {
        let expr = "null == null";

        assert_eq!(evaluate(expr).unwrap(), Value::Bool(true));
    }

    #[test]
    fn null_equals_nothing_else() {
        for val in ["123", "\"string\"", "true", "12.3"] {
            let expr = format!("null == {}", val);

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Bool(false),
                "Null matched {}",
                val
            );

            let expr = format!("{} == null", val);

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Bool(false),
                "Null matched {}",
                val
            );

            let expr = format!("null != {}", val);

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Bool(true),
                "Null matched {}",
                val
            );

            let expr = format!("{} != null", val);

            assert_eq!(
                evaluate(&expr).unwrap(),
                Value::Bool(true),
                "Null matched {}",
                val
            );
        }
    }

    #[test]
    fn unexpected_identifier() {
        let expr = "unknown";

        let mut env = Environment::default();
        env.add_global("alice", Value::String("bob".to_owned()));

        let err = evaluate_with_env(expr, env).unwrap_err();

        assert_eq!(err.to_string(), "Unexpected identifier `unknown`");
    }

    mod variable_access {
        use super::*;

        #[test]
        fn basic() {
            let expr = "@alice";

            let mut env = Environment::default();
            env.add_global("alice", Value::String("bob".to_owned()));

            assert_eq!(
                evaluate_with_env(expr, env).unwrap(),
                Value::String("bob".to_owned())
            );
        }

        #[test]
        fn nested() {
            let expr = "@alice.bob";

            let mut env = Environment::default();
            env.add_global(
                "alice",
                Value::Object(IndexMap::from([(
                    "bob".to_string(),
                    Value::String("eve".to_owned()),
                )])),
            );

            assert_eq!(
                evaluate_with_env(expr, env).unwrap(),
                Value::String("eve".to_owned())
            );
        }

        #[test]
        fn unknown_key_on_object() {
            let expr = "@alice.dontexist";

            let mut env = Environment::default();
            env.add_global(
                "alice",
                Value::Object(IndexMap::from([(
                    "bob".to_string(),
                    Value::String("eve".to_owned()),
                )])),
            );

            let err = evaluate_with_env(expr, env).unwrap_err();

            assert_eq!(
                err.to_string(),
                "Could not find field `dontexist` on `object` `{ bob: \"eve\" }`"
            );
        }

        #[test]
        fn unknown_variable() {
            let expr = "@unknown";

            let mut env = Environment::default();
            env.add_global("alice", Value::String("bob".to_owned()));

            let err = evaluate_with_env(expr, env).unwrap_err();

            assert_eq!(err.to_string(), "Variable `@unknown` not found");
        }

        #[test]
        fn invalid_dot_access() {
            let expr = "@alice.cookie";

            let mut env = Environment::default();
            env.add_global("alice", Value::String("bob".to_owned()));

            let err = evaluate_with_env(expr, env).unwrap_err();

            assert_eq!(
                err.to_string(),
                "Could not find field `cookie` on `string` `\"bob\"`"
            );
        }
    }

    mod capitalize {
        use super::*;

        #[test]
        fn capitalizes_a_string() {
            let expr = "\"foo\" | capitalize";

            assert_eq!(evaluate(expr).unwrap(), Value::String("Foo".to_string()));
        }

        #[test]
        fn validates_argument() {
            let expr = "true | capitalize";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Unexpected argument to filter `capitalize`. Expected a `string` as the first argument, found `true` with type `bool`"
            )
        }

        #[test]
        fn validates_argument_count() {
            let expr = "capitalize()";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Wrong number of arguments for filter `capitalize`. Expected 1 argument(s), found 0"
            );

            let expr = "capitalize(\"foo\", \"foo\")";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Wrong number of arguments for filter `capitalize`. Expected 1 argument(s), found 2"
            );
        }
    }

    mod append {
        use super::*;

        #[test]
        fn appends_to_string() {
            let expr = "\"foo\" | append(\" bar\")";

            assert_eq!(
                evaluate(expr).unwrap(),
                Value::String("foo bar".to_string())
            );
        }

        #[test]
        fn validates_argument_count() {
            let expr = "\"foo\" | append(\"foo\", \"bar\")";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Wrong number of arguments for filter `append`. Expected 2 argument(s), found 3"
            );

            let expr = "\"foo\" | append()";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Wrong number of arguments for filter `append`. Expected 2 argument(s), found 1"
            );
        }

        #[test]
        fn validates_second_argument() {
            let expr = "\"foo\" | append(-23)";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Unexpected argument to filter `append`. Expected a `string` as the second argument, found `-23` with type `number`"
            )
        }

        #[test]
        fn validates_first_argument() {
            let expr = "123 | append(\" foo\")";

            let e = evaluate(expr).unwrap_err();

            assert_eq!(
                &e.to_string(),
                "Unexpected argument to filter `append`. Expected a `string` as the first argument, found `123` with type `number`"
            )
        }
    }
}
