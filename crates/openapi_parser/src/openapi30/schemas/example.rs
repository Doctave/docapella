use crate::openapi30::parser;
use crate::openapi30::parser::ParserContext;
use crate::Set;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Example object should contain either 'value' or 'externalValue', but not both"#)]
    ConflictingValueAndExternalValue,
}

#[derive(Debug, Clone)]
pub struct Example {
    pub summary: Option<String>,
    pub description: Option<String>,
    pub value: Option<Value>,
    pub external_value: Option<String>,
}

impl Example {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let mut value = value
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|ref_path| ctx.get(ref_path, visited_refs))
            .unwrap_or(value);

        let summary = value.take("summary").and_then(Value::take_string);
        let description = value.take("description").and_then(Value::take_string);
        let value_field = value.take("value");
        let external_value = value.take("externalValue").and_then(Value::take_string);

        if value_field.is_some() && external_value.is_some() {
            return Err(Error::ConflictingValueAndExternalValue.into());
        }

        Ok(Example {
            summary,
            description,
            value: value_field,
            external_value,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_example_with_value() {
        let value = json!({
            "summary": "A sample example",
            "description": "This is a sample example with a value",
            "value": {
                "name": "John Doe",
                "age": 30
            }
        });
        let example =
            Example::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            example.pretty_print(),
            indoc! {r#"
            {
              "summary": "A sample example",
              "description": "This is a sample example with a value",
              "value": {
                "name": "John Doe",
                "age": 30
              }
            }"#}
        )
    }

    #[test]
    fn parses_example_with_external_value() {
        let value = json!({
            "summary": "An external example",
            "externalValue": "https://example.com/examples/user-example.json"
        });
        let example =
            Example::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            example.pretty_print(),
            indoc! {r#"
            {
              "summary": "An external example",
              "externalValue": "https://example.com/examples/user-example.json"
            }"#}
        )
    }

    #[test]
    fn parses_example_with_minimal_fields() {
        let value = json!({
            "value": "A simple string example"
        });
        let example =
            Example::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            example.pretty_print(),
            indoc! {r#"
            {
              "value": "A simple string example"
            }"#}
        )
    }

    #[test]
    fn fails_on_conflicting_value_and_external_value() {
        let value = json!({
            "value": "An inline example",
            "externalValue": "https://example.com/examples/example.json"
        });
        let err =
            Example::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Example object should contain either 'value' or 'externalValue', but not both"#
        )
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/examples/Example1"
        });
        let ref_cache = json!({
          "examples": {
            "Example1": {
              "summary": "A sample example",
              "description": "This is a sample example with a value",
              "value": {
                  "name": "John Doe",
                  "age": 30
              }
            }
          }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(ref_cache);

        let example = Example::try_parse(value, &ctx, &mut Set::new()).unwrap();

        assert_str_eq!(
            example.pretty_print(),
            indoc! {r#"
            {
              "summary": "A sample example",
              "description": "This is a sample example with a value",
              "value": {
                "name": "John Doe",
                "age": 30
              }
            }"#}
        )
    }
}
