use crate::String;

use crate::openapi30::parser;
use crate::Map;
use crate::Value;
use thiserror::Error;

use super::server_variable::ServerVariable;

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Server object should contain a url"#)]
    MissingUrl,
}

#[derive(Debug, Clone)]
pub struct Server {
    pub url: String,
    pub description: Option<String>,
    pub variables: Map<String, ServerVariable>,
}

impl Server {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let url = value
            .take("url")
            .and_then(Value::take_string)
            .ok_or(Error::MissingUrl)?;

        let description = value.take("description").and_then(Value::take_string);

        let mut variables = Map::new();

        if let Some(vars) = value.take("variables").and_then(Value::take_object) {
            for (k, v) in vars.into_iter() {
                variables.insert(k, ServerVariable::try_parse(v)?);
            }
        }

        Ok(Server {
            url,
            description,
            variables,
        })
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use crate::{json, openapi30::schemas::server::Server};

    #[test]
    fn parses_server() {
        let value = json!({
            "url": "https://api.example.com/v1",
        });

        let server = Server::try_parse(value).unwrap();

        assert_str_eq!(
            server.pretty_print(),
            indoc! {r#"
            {
              "url": "https://api.example.com/v1"
            }"# }
        )
    }

    #[test]
    fn parses_server_with_description() {
        let value = json!({
            "url": "https://api.example.com/v1",
            "description": "Example server"
        });

        let server = Server::try_parse(value).unwrap();

        assert_str_eq!(
            server.pretty_print(),
            indoc! {r#"
            {
              "url": "https://api.example.com/v1",
              "description": "Example server"
            }"# }
        )
    }

    #[test]
    fn parses_server_with_server_variables() {
        let value = json!({
            "url": "https://api.example.com/v1",
            "variables": {
                "version": {
                    "default": "v1",
                    "enum": ["v1", "v2"],
                    "description": "API version"
                }
            }
        });

        let server = Server::try_parse(value).unwrap();

        assert_str_eq!(
            server.pretty_print(),
            indoc! {r#"
            {
              "url": "https://api.example.com/v1",
              "variables": {
                "version": {
                  "enum": [
                    "v1",
                    "v2"
                  ],
                  "default": "v1",
                  "description": "API version"
                }
              }
            }"# }
        )
    }
}
