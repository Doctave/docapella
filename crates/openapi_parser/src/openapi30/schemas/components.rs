use crate::openapi30::parser::ParserContext;
use crate::Map;
use crate::Set;
use crate::String;
use crate::Value;
use thiserror::Error;

use crate::openapi30::parser;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid"#)]
    Invalid,
}

use super::{
    callback::Callback, example::Example, header::Header, link::Link, parameter::Parameter,
    request_body::RequestBody, response::Response, schema::Schema, security_scheme::SecurityScheme,
};

#[derive(Debug, Clone, Default)]
pub struct Components {
    pub schemas: Map<String, Schema>,                  // referable
    pub responses: Map<String, Response>,              // referable
    pub parameters: Map<String, Parameter>,            // referable
    pub examples: Map<String, Example>,                // referable
    pub request_bodies: Map<String, RequestBody>,      // referable
    pub headers: Map<String, Header>,                  // referable
    pub security_schemes: Map<String, SecurityScheme>, // referable
    pub links: Map<String, Link>,                      // referable
    pub callbacks: Map<String, Callback>,              // referable
}

impl Components {
    pub fn try_parse(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let schemas = value
            .take("schemas")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Schema::try_parse(v, ctx, visited_refs, None).map(|schema| (k, schema))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let responses = value
            .take("responses")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Response::try_parse(v, ctx, visited_refs).map(|response| (k, response))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let parameters = value
            .take("parameters")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Parameter::try_parse(v, ctx, visited_refs).map(|parameter| (k, parameter))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let examples = value
            .take("examples")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Example::try_parse(v, ctx, visited_refs).map(|example| (k, example))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let request_bodies = value
            .take("requestBodies")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        RequestBody::try_parse(v, ctx, visited_refs)
                            .map(|request_body| (k, request_body))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let headers = value
            .take("headers")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Header::try_parse(v, k.clone(), ctx, visited_refs).map(|header| (k, header))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let security_schemes = value
            .take("securitySchemes")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        SecurityScheme::try_parse(v, ctx, visited_refs)
                            .map(|security_scheme| (k, security_scheme))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let links = value
            .take("links")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| Link::try_parse(v, ctx, visited_refs).map(|link| (k, link)))
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let callbacks = value
            .take("callbacks")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Callback::try_parse(v, ctx, visited_refs).map(|callback| (k, callback))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            schemas,
            responses,
            parameters,
            examples,
            request_bodies,
            headers,
            security_schemes,
            links,
            callbacks,
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;

    #[test]
    fn parses_schemas() {
        let value = json!({
          "schemas": {
            "User": {
              "type": "object",
              "properties": {
                "id": {
                  "type": "number"
                },
                "name": {
                  "type": "string"
                }
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.schemas.get("User").is_some());
    }

    #[test]
    #[ignore]
    fn parses_responses() {
        let value = json!({
          "responses": {
            "NotFound": {
              "description": "Resource not found"
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.responses.get("NotFound").is_some());
    }

    #[test]
    #[ignore]
    fn parses_parameters() {
        let value = json!({
          "parameters": {
            "Id": {
              "name": "id",
              "in": "path",
              "required": true,
              "schema": {
                "type": "number"
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.parameters.get("Id").is_some());
    }

    #[test]
    #[ignore]
    fn parses_examples() {
        let value = json!({
          "examples": {
            "User": {
              "summary": "An example user",
              "value": {
                "id": 1,
                "name": "Alice"
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.examples.get("User").is_some());
    }

    #[test]
    #[ignore]
    fn parses_request_bodies() {
        let value = json!({
          "requestBodies": {
            "User": {
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "number"
                      },
                      "name": {
                        "type": "string"
                      }
                    }
                  }
                }
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.request_bodies.get("User").is_some());
    }

    #[test]
    #[ignore]
    fn parses_headers() {
        let value = json!({
          "headers": {
            "X-Rate-Limit": {
              "description": "The number of allowed requests in the current period",
              "schema": {
                "type": "number"
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.headers.get("X-Rate-Limit").is_some());
    }

    #[test]
    #[ignore]
    fn parses_security_schemes() {
        let value = json!({
          "securitySchemes": {
            "ApiKey": {
              "type": "apiKey",
              "name": "api_key",
              "in": "header"
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.security_schemes.get("ApiKey").is_some());
    }

    #[test]
    #[ignore]
    fn parses_links() {
        let value = json!({
          "links": {
            "User": {
              "operationId": "getUser",
              "parameters": {
                "userId": "$response.body#/id"
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.links.get("User").is_some());
    }

    #[test]
    #[ignore]
    fn parses_callbacks() {
        let value = json!({
          "callbacks": {
            "User": {
              "getUser": {
                "summary": "A callback to get a user",
                "operationId": "getUser",
                "parameters": {
                  "userId": "$response.body#/id"
                }
              }
            }
          }
        });

        let components =
            Components::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();

        assert!(components.callbacks.get("User").is_some());
    }
}
