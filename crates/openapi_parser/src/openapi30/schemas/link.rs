use super::server::Server;
use crate::openapi30::parser;
use crate::openapi30::parser::ParserContext;
use crate::Map;
use crate::Set;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Link object should contain either operationRef or operationId"#)]
    MissingOperation,
}

#[derive(Debug, Clone)]
pub struct Link {
    pub operation_ref: Option<String>,
    pub operation_id: Option<String>,
    pub parameters: Map<String, Value>,
    pub request_body: Option<Value>,
    pub description: Option<String>,
    pub server: Option<Server>,
}

impl Link {
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

        let operation_ref = value.take("operationRef").and_then(Value::take_string);
        let operation_id = value.take("operationId").and_then(Value::take_string);

        if operation_ref.is_none() && operation_id.is_none() {
            return Err(Error::MissingOperation.into());
        }

        let parameters = value
            .take("parameters")
            .and_then(|v| v.take_object())
            .unwrap_or_default();
        let request_body = value.take("requestBody");
        let description = value.take("description").and_then(Value::take_string);
        let server = value.take("server").and_then(|s| Server::try_parse(s).ok());

        Ok(Link {
            operation_ref,
            operation_id,
            parameters,
            request_body,
            description,
            server,
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
    fn parses_link_with_operation_ref() {
        let value = json!({
            "operationRef": "#/paths/~1users~1{userId}/get"
        });
        let link = Link::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            link.pretty_print(),
            indoc! {r##"
            {
              "operationRef": "#/paths/~1users~1{userId}/get"
            }"##}
        )
    }

    #[test]
    fn parses_link_with_operation_id() {
        let value = json!({
            "operationId": "getUserById"
        });
        let link = Link::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            link.pretty_print(),
            indoc! {r#"
            {
              "operationId": "getUserById"
            }"#}
        )
    }

    #[test]
    fn parses_link_with_parameters() {
        let value = json!({
            "operationId": "getUserById",
            "parameters": {
                "userId": "$request.path.id"
            }
        });
        let link = Link::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            link.pretty_print(),
            indoc! {r#"
            {
              "operationId": "getUserById",
              "parameters": {
                "userId": "$request.path.id"
              }
            }"#}
        )
    }

    #[test]
    fn parses_link_with_description() {
        let value = json!({
            "operationId": "getUserById",
            "description": "Get user information"
        });
        let link = Link::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            link.pretty_print(),
            indoc! {r#"
            {
              "operationId": "getUserById",
              "description": "Get user information"
            }"#}
        )
    }

    #[test]
    fn missing_operation() {
        let value = json!({
            "parameters": {
                "userId": "$request.path.id"
            }
        });
        let err = Link::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Link object should contain either operationRef or operationId"#
        )
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/links/GetUserById"
        });

        let ref_cache = json!({
          "links": {
            "GetUserById": {
                "operationId": "getUserById"
            }
          }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(ref_cache);

        let link = Link::try_parse(value, &ctx, &mut Set::new()).unwrap();
        assert_str_eq!(
            link.pretty_print(),
            indoc! {r#"
            {
              "operationId": "getUserById"
            }"#}
        )
    }
}
