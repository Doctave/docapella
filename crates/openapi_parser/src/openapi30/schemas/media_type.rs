use super::encoding::Encoding;
use super::example::Example;
use super::schema::Schema;
use crate::openapi30::parser::{self, ParserContext};
use crate::{Map, Set};
use crate::{String, Value};

#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Failed to parse schema: {0}"#)]
    SchemaParseError(String),
    #[error(r#"Failed to parse example: {0}"#)]
    ExampleParseError(String),
    #[error(r#"Failed to parse encoding: {0}"#)]
    EncodingParseError(String),
}

#[derive(Debug, Clone)]
pub struct MediaType {
    pub schema: Option<Schema>,
    pub example: Option<Value>,
    pub examples: Map<String, Example>,
    pub encoding: Map<String, Encoding>,
}

impl MediaType {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let schema = value
            .take("schema")
            .map(|v| Schema::try_parse(v, ctx, visited_refs, None))
            .transpose()?;

        let example = value.take("example");

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

        let encoding = value
            .take("encoding")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Encoding::try_parse(v, ctx, visited_refs).map(|encoding| (k, encoding))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(MediaType {
            schema,
            example,
            examples,
            encoding,
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
    fn parses_schema() {
        let value = json!({
            "schema": {
                "type": "object",
                "properties": {
                    "id": { "type": "number" },
                    "name": { "type": "string" }
                }
            }
        });
        let media_type =
            MediaType::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            media_type.pretty_print(),
            indoc! {r#"
            {
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
            }"#}
        );
    }

    #[test]
    fn parses_example() {
        let value = json!({
            "example": {
                "id": 1,
                "name": "Example"
            }
        });
        let media_type =
            MediaType::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            media_type.pretty_print(),
            indoc! {r#"
            {
              "example": {
                "id": 1,
                "name": "Example"
              }
            }"#}
        );
    }

    #[test]
    fn parses_examples() {
        let value = json!({
            "examples": {
                "example1": {
                    "value": {
                        "id": 1,
                        "name": "Example 1"
                    }
                },
                "example2": {
                    "value": {
                        "id": 2,
                        "name": "Example 2"
                    }
                }
            }
        });
        let media_type =
            MediaType::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            media_type.pretty_print(),
            indoc! {r#"
            {
              "examples": {
                "example1": {
                  "value": {
                    "id": 1,
                    "name": "Example 1"
                  }
                },
                "example2": {
                  "value": {
                    "id": 2,
                    "name": "Example 2"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_encoding() {
        let value = json!({
            "encoding": {
                "historyMetadata": {
                    "contentType": "application/xml; charset=utf-8"
                }
            }
        });
        let media_type =
            MediaType::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            media_type.pretty_print(),
            indoc! {r#"
            {
              "encoding": {
                "historyMetadata": {
                  "contentType": "application/xml; charset=utf-8"
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_all_fields() {
        let value = json!({
            "schema": {
                "type": "object",
                "properties": {
                    "id": { "type": "number" },
                    "name": { "type": "string" }
                }
            },
            "example": {
                "id": 1,
                "name": "Example"
            },
            "examples": {
                "example1": {
                    "value": {
                        "id": 1,
                        "name": "Example 1"
                    }
                }
            },
            "encoding": {
                "historyMetadata": {
                    "contentType": "application/xml; charset=utf-8"
                }
            }
        });
        let media_type =
            MediaType::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            media_type.pretty_print(),
            indoc! {r#"
            {
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
              },
              "example": {
                "id": 1,
                "name": "Example"
              },
              "examples": {
                "example1": {
                  "value": {
                    "id": 1,
                    "name": "Example 1"
                  }
                }
              },
              "encoding": {
                "historyMetadata": {
                  "contentType": "application/xml; charset=utf-8"
                }
              }
            }"#}
        );
    }
}
