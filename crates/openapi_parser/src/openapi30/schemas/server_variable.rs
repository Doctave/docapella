use crate::String;

use crate::openapi30::parser;
use crate::Value;
use thiserror::Error;

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"ServerVariable object should contain a default"#)]
    MissingDefault,
}

#[derive(Debug, Clone)]
pub struct ServerVariable {
    pub r#enum: Option<Vec<String>>,
    pub default: String,
    pub description: Option<String>,
}

impl ServerVariable {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let default = value
            .take("default")
            .and_then(Value::take_string)
            .ok_or(Error::MissingDefault)?;

        let description = value.take("description").and_then(Value::take_string);
        let r#enum = value
            .take("enum")
            .and_then(Value::take_array)
            .map(|arr| arr.into_iter().filter_map(Value::take_string).collect());

        Ok(ServerVariable {
            default,
            description,
            r#enum,
        })
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use crate::{json, openapi30::schemas::server_variable::ServerVariable};

    #[test]
    fn parses_server_variable() {
        let value = json!({
            "default": "v1"
        });

        let server_variable = ServerVariable::try_parse(value).unwrap();

        assert_str_eq!(
            server_variable.pretty_print(),
            indoc! {r#"
          {
            "default": "v1"
          }"# }
        )
    }

    #[test]
    fn parses_server_variable_with_description() {
        let value = json!({
            "default": "v1",
            "description": "Version"
        });

        let server_variable = ServerVariable::try_parse(value).unwrap();

        assert_str_eq!(
            server_variable.pretty_print(),
            indoc! {r#"
          {
            "default": "v1",
            "description": "Version"
          }"# }
        )
    }

    #[test]
    fn parses_server_variable_with_enum() {
        let value = json!({
            "default": "v1",
            "description": "Version",
            "enum": ["v1", "v2"]
        });

        let server_variable = ServerVariable::try_parse(value).unwrap();

        assert_str_eq!(
            server_variable.pretty_print(),
            indoc! {r#"
          {
            "enum": [
              "v1",
              "v2"
            ],
            "default": "v1",
            "description": "Version"
          }"# }
        )
    }

    #[test]
    fn missing_default() {
        let value = json!({
            "description": "Version",
            "enum": ["v1", "v2"],
        });

        let err = ServerVariable::try_parse(value).unwrap_err();

        assert_str_eq!(
            err.to_string(),
            r#"ServerVariable object should contain a default"#
        )
    }
}
