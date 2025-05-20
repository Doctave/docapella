/// Filter registry for expressions.
///
/// To create a new filter, add it to the static `FILTERS` list. That is used at runtime to look
/// up filters and execute them.
///
/// Filters are responsible for their own error handling and have to return meaningful errors.
use super::{interpreter::Value, Error, Result, Span};

pub(crate) type ApplyFilter = fn(args: Vec<Value>, pos: Span) -> Result<Value>;

#[derive(Debug)]
/// A filter function that can be invoked inside an expression
pub(crate) struct Filter {
    name: &'static str,
    apply: ApplyFilter,
}

impl Filter {
    fn new(name: &'static str, apply: ApplyFilter) -> Self {
        Filter { name, apply }
    }
}

pub(crate) fn apply_filter(name: &str, args: Vec<Value>, pos: Span) -> Result<Value> {
    if let Some(filter) = FILTERS.iter().find(|f| f.name == name) {
        (filter.apply)(args, pos)
    } else {
        Err(Error::UnknownFilter(
            name.to_string(),
            Span::new(pos.start, pos.end),
        ))
    }
}

lazy_static! {
    static ref FILTERS: Vec<Filter> = vec![
        Filter::new(
            "capitalize",
            |args: Vec<Value>, pos: Span| -> Result<Value> {
                if args.len() != 1 {
                    return Err(Error::InvalidArity(
                        "capitalize",
                        1,
                        args.len(),
                        Span::new(pos.start, pos.end),
                    ));
                }

                let arg = args.first().unwrap();

                if let Value::String(s) = arg {
                    let mut c = s.chars();
                    Ok(Value::String(match c.next() {
                        None => String::new(),
                        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                    }))
                } else {
                    Err(Error::InvalidArgument {
                        filter: "capitalize",
                        expected_type: "string",
                        argument_index: 0,
                        actual_value: arg.debug_string(),
                        actual_type: arg.type_name().to_owned(),
                        span: Span::new(pos.start, pos.start),
                    })
                }
            }
        ),
        Filter::new("append", |args: Vec<Value>, pos: Span| -> Result<Value> {
            if args.len() != 2 {
                return Err(Error::InvalidArity(
                    "append",
                    2,
                    args.len(),
                    Span::new(pos.start, pos.end),
                ));
            }

            if let [Value::String(front), Value::String(back)] = &args[0..2] {
                Ok(Value::String(format!("{}{}", front, back)))
            } else if matches!(args.first(), Some(Value::String(_))) {
                let other = args.get(1).unwrap();

                Err(Error::InvalidArgument {
                    filter: "append",
                    expected_type: "string",
                    argument_index: 1,
                    actual_value: other.debug_string(),
                    actual_type: other.type_name().to_owned(),
                    span: Span::new(pos.start, pos.start),
                })
            } else {
                let other = args.first().unwrap();

                Err(Error::InvalidArgument {
                    filter: "append",
                    expected_type: "string",
                    argument_index: 0,
                    actual_value: other.debug_string(),
                    actual_type: other.type_name().to_owned(),
                    span: Span::new(pos.start, pos.start),
                })
            }
        }),
    ];
}
