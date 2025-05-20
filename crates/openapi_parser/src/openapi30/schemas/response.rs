use super::header::Header;
use super::link::Link;
use super::media_type::MediaType;
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
    #[error(r#"Response object should contain a description"#)]
    MissingDescription,
    #[error(r#"Failed to parse header: {0}"#)]
    HeaderParseError(String),
    #[error(r#"Failed to parse media type: {0}"#)]
    MediaTypeParseError(String),
    #[error(r#"Failed to parse link: {0}"#)]
    LinkParseError(String),
}

#[derive(Debug, Clone)]
pub struct Response {
    pub description: String,
    pub headers: Map<String, Header>,
    pub content: Map<String, MediaType>,
    pub links: Map<String, Link>,
}

impl Response {
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
        let mut visited_refs = visited_refs.clone();
        let mut value = value
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|ref_path| ctx.get(ref_path, &mut visited_refs))
            .unwrap_or(value);

        let description = value
            .take("description")
            .and_then(Value::take_string)
            .ok_or(Error::MissingDescription)?;

        let headers = value
            .take("headers")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .filter_map(|(k, v)| {
                        if k == "Content-Type" {
                            None
                        } else {
                            Some(
                                Header::try_parse(v, k.clone(), ctx, &mut visited_refs)
                                    .map(|h| (k, h)),
                            )
                        }
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let content = value
            .take("content")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        MediaType::try_parse(v, ctx, &mut visited_refs)
                            .map(|media_type| (k, media_type))
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
                    .map(|(k, v)| Link::try_parse(v, ctx, &mut visited_refs).map(|link| (k, link)))
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Response {
            description,
            headers,
            content,
            links,
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
    fn parses_description() {
        let value = json!({
            "description": "A sample response"
        });
        let response =
            Response::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            response.pretty_print(),
            indoc! {r#"
            {
              "description": "A sample response"
            }"#}
        );
    }

    #[test]
    fn parses_headers() {
        let value = json!({
            "description": "A sample response",
            "headers": {
                "X-Rate-Limit-Limit": {
                    "description": "The number of allowed requests in the current period",
                    "schema": {
                        "type": "number"
                    }
                }
            }
        });
        let response =
            Response::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            response.pretty_print(),
            indoc! {r#"
            {
              "description": "A sample response",
              "headers": {
                "X-Rate-Limit-Limit": {
                  "name": "X-Rate-Limit-Limit",
                  "in": "header",
                  "schema": {
                    "description": "The number of allowed requests in the current period",
                    "type": "number",
                    "doctave_metadata": {
                      "field_name": "X-Rate-Limit-Limit"
                    }
                  },
                  "description": "The number of allowed requests in the current period"
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_content() {
        let value = json!({
            "description": "A sample response",
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "number" },
                            "name": { "type": "string" }
                        }
                    }
                }
            }
        });
        let response =
            Response::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            response.pretty_print(),
            indoc! {r#"
            {
              "description": "A sample response",
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "number",
                        "doctave_metadata": {
                          "field_name": "id"
                        }
                      },
                      "name": {
                        "type": "string",
                        "doctave_metadata": {
                          "field_name": "name"
                        }
                      }
                    }
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_links() {
        let value = json!({
            "description": "A sample response",
            "links": {
                "UserProfile": {
                    "operationId": "getUserProfile",
                    "parameters": {
                        "userId": "$response.body#/id"
                    }
                }
            }
        });
        let response =
            Response::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            response.pretty_print(),
            indoc! {r#"
            {
              "description": "A sample response",
              "links": {
                "UserProfile": {
                  "operationId": "getUserProfile",
                  "parameters": {
                    "userId": "$response.body#/id"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn fails_on_missing_description() {
        let value = json!({
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object"
                    }
                }
            }
        });
        let err =
            Response::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Response object should contain a description"#
        );
    }

    #[test]
    fn parses_all_fields() {
        let value = json!({
            "description": "A sample response",
            "headers": {
                "X-Rate-Limit-Limit": {
                    "description": "The number of allowed requests in the current period",
                    "schema": {
                        "type": "number",
                    }
                }
            },
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "number" },
                            "name": { "type": "string" }
                        }
                    }
                }
            },
            "links": {
                "UserProfile": {
                    "operationId": "getUserProfile",
                    "parameters": {
                        "userId": "$response.body#/id"
                    }
                }
            }
        });
        let response =
            Response::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            response.pretty_print(),
            indoc! {r#"
            {
              "description": "A sample response",
              "headers": {
                "X-Rate-Limit-Limit": {
                  "name": "X-Rate-Limit-Limit",
                  "in": "header",
                  "schema": {
                    "description": "The number of allowed requests in the current period",
                    "type": "number",
                    "doctave_metadata": {
                      "field_name": "X-Rate-Limit-Limit"
                    }
                  },
                  "description": "The number of allowed requests in the current period"
                }
              },
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "number",
                        "doctave_metadata": {
                          "field_name": "id"
                        }
                      },
                      "name": {
                        "type": "string",
                        "doctave_metadata": {
                          "field_name": "name"
                        }
                      }
                    }
                  }
                }
              },
              "links": {
                "UserProfile": {
                  "operationId": "getUserProfile",
                  "parameters": {
                    "userId": "$response.body#/id"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/responses/NotFound"
        });

        let ref_cache = json!({
          "responses": {
            "NotFound": {
                "description": "The resource was not found"
            }
          }
        });

        let mut context = ParserContext::default();

        context.ref_cache.with_value(ref_cache);

        let response = Response::try_parse(value, &context, &mut Set::new()).unwrap();

        assert_str_eq!(
            response.pretty_print(),
            indoc! {r#"
            {
              "description": "The resource was not found"
            }"#}
        );
    }
}
