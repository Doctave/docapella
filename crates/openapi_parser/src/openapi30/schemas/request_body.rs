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
    #[error(r#"RequestBody object should contain a content field"#)]
    MissingContent,
    #[error(r#"Failed to parse MediaType: {0}"#)]
    MediaTypeParseError(String),
}

#[derive(Debug, Clone)]
pub struct RequestBody {
    pub description: Option<String>,
    pub content: Map<String, MediaType>,
    pub required: Option<bool>,
}

impl RequestBody {
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

        let description = value.take("description").and_then(Value::take_string);
        let required = value.take("required").and_then(Value::take_bool);

        let content = value
            .take("content")
            .and_then(Value::take_object)
            .ok_or(Error::MissingContent)?
            .into_iter()
            .map(|(k, v)| {
                MediaType::try_parse(v, ctx, &mut visited_refs).map(|media_type| (k, media_type))
            })
            .collect::<Result<Map<_, _>, _>>()?;

        if content.is_empty() {
            return Err(Error::MissingContent.into());
        }

        Ok(RequestBody {
            description,
            content,
            required,
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
    fn parses_single_content_type() {
        let value = json!({
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        }
                    }
                }
            }
        });
        let request_body =
            RequestBody::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            request_body.pretty_print(),
            indoc! {r#"
            {
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
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
    fn parses_multiple_content_types() {
        let value = json!({
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        }
                    }
                },
                "application/xml": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        }
                    }
                }
            }
        });
        let request_body =
            RequestBody::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            request_body.pretty_print(),
            indoc! {r#"
            {
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
                      "name": {
                        "type": "string",
                        "doctave_metadata": {
                          "field_name": "name"
                        }
                      }
                    }
                  }
                },
                "application/xml": {
                  "schema": {
                    "type": "object",
                    "properties": {
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
    fn parses_description() {
        let value = json!({
            "description": "User object that needs to be added to the store",
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        }
                    }
                }
            }
        });
        let request_body =
            RequestBody::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            request_body.pretty_print(),
            indoc! {r#"
            {
              "description": "User object that needs to be added to the store",
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
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
    fn parses_required() {
        let value = json!({
            "required": true,
            "content": {
                "application/json": {
                    "schema": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" }
                        }
                    }
                }
            }
        });
        let request_body =
            RequestBody::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            request_body.pretty_print(),
            indoc! {r#"
            {
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
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
              "required": true
            }"#}
        );
    }

    #[test]
    fn fails_on_missing_content() {
        let value = json!({
            "description": "User object that needs to be added to the store"
        });
        let err =
            RequestBody::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"RequestBody object should contain a content field"#
        );
    }

    #[test]
    fn fails_on_empty_content() {
        let value = json!({
            "content": {}
        });
        let err =
            RequestBody::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"RequestBody object should contain a content field"#
        );
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
          "$ref": "#/components/request_bodies/User"
        });

        let ref_cache = json!({
            "request_bodies": {
              "User": {
                  "description": "User object that needs to be added to the store",
                  "content": {
                      "application/json": {
                          "schema": {
                              "type": "object",
                              "properties": {
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

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(ref_cache);

        let request_body = RequestBody::try_parse(value, &ctx, &mut Set::new()).unwrap();
        assert_str_eq!(
            request_body.pretty_print(),
            indoc! {r##"
            {
              "description": "User object that needs to be added to the store",
              "content": {
                "application/json": {
                  "schema": {
                    "type": "object",
                    "properties": {
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
            }"##}
        );
    }
}
