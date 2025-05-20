use crate::{
    openapi30::parser::{self, ParserContext},
    Set, String, Value,
};

use crate::Map;
#[cfg(test)]
use serde_json::to_string_pretty;

use super::{
    callback::Callback, external_documentation::ExternalDocumentation, parameter::Parameter,
    request_body::RequestBody, responses::Responses, security_requirement::SecurityRequirement,
    server::Server,
};

#[derive(Debug, Clone)]
pub struct Operation {
    pub method: String,
    pub route_pattern: String,
    pub tags: Vec<String>,
    pub summary: Option<String>,
    pub description: Option<String>,
    pub external_docs: Option<ExternalDocumentation>,
    pub operation_id: Option<String>,
    pub responses: Option<Responses>,
    pub deprecated: Option<bool>,
    pub security: Vec<SecurityRequirement>,
    pub servers: Vec<Server>,
    pub extensions: Map<String, Value>,

    // referables, should check from ref cache
    pub parameters: Vec<Parameter>,        // referable
    pub request_body: Option<RequestBody>, // referable
    pub callbacks: Map<String, Callback>,  // referable
}

impl Operation {
    #[cfg(test)]
    pub(crate) fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
        method: String,
        route_pattern: String,
        path_params: Vec<Parameter>,
        path_or_root_servers: &[Server],
    ) -> parser::Result<Self> {
        let summary = value.take("summary").and_then(Value::take_string);
        let description = value.take("description").and_then(Value::take_string);

        let tags: Vec<String> = value
            .take("x-doctave-tags")
            .or(value.take("tags"))
            .and_then(Value::take_array)
            .map(|v| v.into_iter().filter_map(|v| v.take_string()).collect())
            .unwrap_or_default();

        let operation_id = value.take("operationId").and_then(Value::take_string);
        let deprecated = value.take("deprecated").and_then(Value::take_bool);

        let mut servers = vec![];
        if let Some(_servers) = value.take("servers").and_then(Value::take_array) {
            for server in _servers {
                servers.push(Server::try_parse(server)?)
            }
        } else {
            servers = path_or_root_servers.to_vec();
        }

        let responses = value
            .take("responses")
            .map(|v| Responses::try_parse(v, ctx, visited_refs))
            .transpose()?;

        let external_docs = value
            .take("externalDocs")
            .map(ExternalDocumentation::try_parse)
            .transpose()?;

        let mut parameters = path_params;

        let op_parameters = value
            .take("parameters")
            .and_then(Value::take_array)
            .map(|v| {
                v.into_iter()
                    .map(|p| Parameter::try_parse(p, ctx, visited_refs))
                    .collect::<parser::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();

        parameters.extend(op_parameters);

        let request_body = value
            .take("requestBody")
            .map(|v| RequestBody::try_parse(v, ctx, visited_refs))
            .transpose()?;

        let mut callbacks = Map::new();

        if let Some(cbs) = value.take("callbacks").and_then(Value::take_object) {
            for (k, v) in cbs.into_iter() {
                callbacks.insert(k, Callback::try_parse(v, ctx, visited_refs)?);
            }
        }

        let security = value
            .take("security")
            .and_then(Value::take_array)
            .map(|v| {
                v.into_iter()
                    .map(SecurityRequirement::try_parse)
                    .collect::<parser::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();

        let extensions = if let Some(obj) = value.take_object() {
            obj.into_iter()
                .filter(|(k, _)| k.starts_with("x-"))
                .collect::<Map<_, _>>()
        } else {
            Map::new()
        };

        Ok(Self {
            method,
            route_pattern,
            tags,
            summary,
            description,
            external_docs,
            operation_id,
            parameters,
            request_body,
            responses,
            callbacks,
            deprecated,
            security,
            servers,
            extensions,
        })
    }
}

#[cfg(test)]
mod test {
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use super::*;
    use crate::json;

    #[test]
    fn parses_summary() {
        let value = json!({
            "summary": "Get a user"
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "summary": "Get a user",
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_description() {
        let value = json!({
            "description": "Get a user"
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "description": "Get a user",
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_tags() {
        let value = json!({
            "tags": ["user"]
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "tags": [
                "user"
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_x_doctave_tags_as_tags() {
        let value = json!({
            "x-doctave-tags": ["team"]
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "tags": [
                "team"
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn prioritizes_x_doctave_tags_over_tags() {
        let value = json!({
            "tags": ["user"],
            "x-doctave-tags": ["team"]
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "tags": [
                "team"
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_operation_id() {
        let value = json!({
            "operationId": "getUser"
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "operationId": "getUser",
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_deprecated() {
        let value = json!({
            "deprecated": true
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "deprecated": true,
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_servers() {
        let value = json!({
            "servers": [
                {
                    "url": "https://api.example.com/v1",
                    "description": "Production server"
                }
            ]
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "servers": [
                {
                  "url": "https://api.example.com/v1",
                  "description": "Production server"
                }
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_responses() {
        let value = json!({
            "responses": {
                "200": {
                    "description": "A user"
                }
            }
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "responses": {
                "200": {
                  "description": "A user"
                }
              },
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_external_docs() {
        let value = json!({
            "externalDocs": {
                "url": "https://example.com/docs",
                "description": "API documentation"
            }
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "externalDocs": {
                "url": "https://example.com/docs",
                "description": "API documentation"
              },
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_parameters() {
        let value = json!({
            "parameters": [
                {
                    "name": "id",
                    "in": "path",
                    "required": true,
                    "schema": {
                        "type": "string"
                    }
                }
            ]
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "parameters": [
                {
                  "name": "id",
                  "in": "path",
                  "schema": {
                    "type": "string",
                    "doctave_metadata": {
                      "field_name": "id"
                    }
                  },
                  "required": true
                }
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_request_body() {
        let value = json!({
            "requestBody": {
                "content": {
                    "application/json": {
                        "schema": {
                            "type": "object"
                        }
                    }
                }
            }
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "requestBody": {
                "content": {
                  "application/json": {
                    "schema": {
                      "type": "object"
                    }
                  }
                }
              },
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_callbacks() {
        let value = json!({
            "callbacks": {
                "user": {
                    "https://example.com/callback": {
                        "post": {
                            "description": "A user"
                        }
                    }
                }
            }
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "callbacks": {
                "user": {
                  "https://example.com/callback": {
                    "post": {
                      "description": "A user",
                      "method": "post"
                    }
                  }
                }
              },
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_security() {
        let value = json!({
            "security": [
                {
                    "api_key": []
                }
            ]
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "security": [
                {
                  "api_key": []
                }
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn parses_extensions() {
        let value = json!({
            "x-internal-id": "1234"
        });

        let operation = Operation::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "method": "get",
              "x-internal-id": "1234"
            }"#}
        );
    }

    #[test]
    fn uses_servers_from_parent() {
        let server = Server::try_parse(json!(
          {
              "url": "https://api.example.com/v1",
              "description": "Production server"
          }
        ))
        .unwrap();

        let operation = Operation::try_parse(
            json!({}),
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[server],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "servers": [
                {
                  "url": "https://api.example.com/v1",
                  "description": "Production server"
                }
              ],
              "method": "get"
            }"#}
        );
    }

    #[test]
    fn overwrites_parent_servers() {
        let server = Server::try_parse(json!(
          {
              "url": "https://api.example.com/v1",
              "description": "Production server"
          }
        ))
        .unwrap();

        let operation = Operation::try_parse(
            json!({
                "servers": [
                    {
                        "url": "https://api.example.com/v2",
                        "description": "Staging server"
                    }
                ]
            }),
            &ParserContext::default(),
            &mut Set::new(),
            "get".into(),
            "/path".into(),
            vec![],
            &[server],
        )
        .unwrap();

        assert_str_eq!(
            operation.pretty_print().as_str(),
            indoc! {r#"
            {
              "servers": [
                {
                  "url": "https://api.example.com/v2",
                  "description": "Staging server"
                }
              ],
              "method": "get"
            }"#}
        );
    }
}
