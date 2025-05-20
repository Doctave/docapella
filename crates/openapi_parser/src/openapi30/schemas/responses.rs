use super::response::Response;
use crate::openapi30::parser::{self, ParserContext};
use crate::{Map, Set};
use crate::{String, Value};

#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Failed to parse response: {0}"#)]
    ResponseParseError(String),
    #[error(r#"Responses object should contain at least one response or a default response"#)]
    EmptyResponses,
}

#[derive(Debug, Clone)]
pub struct Responses(pub Map<String, Response>);

impl Responses {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let mut responses = Map::new();

        for (key, val) in value.take_object().unwrap_or_default() {
            let response = Response::try_parse(val, ctx, visited_refs)?;
            responses.insert(key, response);
        }

        Ok(Responses(responses))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_default_response() {
        let value = json!({
            "default": {
                "description": "Default response"
            }
        });
        let responses =
            Responses::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            responses.pretty_print(),
            indoc! {r#"
            {
              "default": {
                "description": "Default response"
              }
            }"#}
        );
    }

    #[test]
    fn parses_specific_response() {
        let value = json!({
            "200": {
                "description": "Successful response"
            }
        });
        let responses =
            Responses::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            responses.pretty_print(),
            indoc! {r#"
            {
              "200": {
                "description": "Successful response"
              }
            }"#}
        );
    }

    #[test]
    fn parses_multiple_responses() {
        let value = json!({
            "200": {
                "description": "Successful response"
            },
            "400": {
                "description": "Bad request"
            },
            "default": {
                "description": "Unexpected error"
            }
        });
        let responses =
            Responses::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            responses.pretty_print(),
            indoc! {r#"
            {
              "200": {
                "description": "Successful response"
              },
              "400": {
                "description": "Bad request"
              },
              "default": {
                "description": "Unexpected error"
              }
            }"#}
        );
    }
}
