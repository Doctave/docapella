use super::example::Example;
use super::schema::Overrides;
use super::schema::Schema;
use crate::openapi30::parser;
use crate::openapi30::parser::ParserContext;
use crate::Map;
use crate::MediaType;
use crate::Set;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Parameter object should contain a name"#)]
    MissingName,
    #[error(r#"Parameter object should contain an 'in' field"#)]
    MissingIn,
    #[error(r#"Invalid 'in' value: {0}"#)]
    InvalidIn(String),
    #[error(r#"'required' must be true for path parameters"#)]
    PathParameterNotRequired,
}

#[derive(Debug, Clone)]
pub enum SchemaOrContent {
    Schema(Schema),
    Content(Map<String, MediaType>),
}

#[derive(Debug, Clone)]
pub struct Parameter {
    pub name: String,
    pub kind: ParameterKind,
    pub description: Option<String>,
    pub deprecated: Option<bool>,
    pub style: Option<String>,
    pub explode: Option<bool>,
    pub schema_or_content: Option<SchemaOrContent>,
    pub example: Option<Value>,
    pub examples: Map<String, Example>,
    pub required: Option<bool>,
}

#[derive(Debug, Clone)]
pub enum ParameterKind {
    Header(HeaderParameter),
    Path(PathParameter),
    Query(QueryParameter),
    Cookie(CookieParameter),
}

impl ParameterKind {
    pub fn get_type(&self) -> String {
        match &self {
            ParameterKind::Query(_) => "query".into(),
            ParameterKind::Path(_) => "path".into(),
            ParameterKind::Cookie(_) => "cookie".into(),
            ParameterKind::Header(_) => "header".into(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct HeaderParameter {}

#[derive(Debug, Clone)]
pub struct PathParameter {}

#[derive(Debug, Clone)]
pub struct QueryParameter {
    pub allow_empty_value: Option<bool>,
    pub allow_reserved: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct CookieParameter {}

impl Parameter {
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

        let name = value
            .take("name")
            .and_then(Value::take_string)
            .ok_or(Error::MissingName)?;

        let r#in = value
            .take("in")
            .and_then(Value::take_string)
            .ok_or(Error::MissingIn)?;

        let description = value.take("description").and_then(Value::take_string);
        let deprecated = value.take("deprecated").and_then(Value::take_bool);
        let style = value.take("style").and_then(Value::take_string);
        let explode = value.take("explode").and_then(Value::take_bool);

        let schema_or_content = if let Some(schema_value) = value.take("schema") {
            let mut schema = Schema::try_parse(
                schema_value,
                ctx,
                &mut visited_refs,
                Some(Overrides {
                    description: description.clone(),
                }),
            )?;

            schema.metadata.field_name = Some(name.clone());

            Some(SchemaOrContent::Schema(schema))
        } else if let Some(content_value) = value.take("content") {
            content_value
                .take_object()
                .map(|obj| {
                    obj.into_iter()
                        .map(|(k, v)| {
                            MediaType::try_parse(v, ctx, &mut visited_refs)
                                .map(|media_type| (k, media_type))
                        })
                        .collect::<Result<Map<_, _>, _>>()
                })
                .transpose()?
                .map(SchemaOrContent::Content)
        } else {
            None
        };

        let example = value.take("example");

        let examples = value
            .take("examples")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Example::try_parse(v, ctx, &mut visited_refs).map(|example| (k, example))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()
            .ok()
            .flatten()
            .unwrap_or_default();

        let mut required = value.take("required").and_then(Value::take_bool);

        let kind = match r#in.as_str() {
            "header" => ParameterKind::Header(HeaderParameter {}),
            "path" => {
                if required != Some(true) {
                    required = Some(true);
                }

                ParameterKind::Path(PathParameter {})
            }
            "query" => {
                let allow_empty_value = value.take("allowEmptyValue").and_then(Value::take_bool);
                let allow_reserved = value.take("allowReserved").and_then(Value::take_bool);
                ParameterKind::Query(QueryParameter {
                    allow_empty_value,
                    allow_reserved,
                })
            }
            "cookie" => ParameterKind::Cookie(CookieParameter {}),
            _ => return Err(Error::InvalidIn(r#in).into()),
        };

        Ok(Parameter {
            name,
            kind,
            description,
            deprecated,
            style,
            explode,
            schema_or_content,
            example,
            examples,
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
    fn parses_path_parameter() {
        let value = json!({
            "name": "userId",
            "in": "path"
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "userId",
              "in": "path",
              "required": true
            }"#}
        )
    }

    #[test]
    fn parses_query_parameter() {
        let value = json!({
            "name": "offset",
            "in": "query"
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query"
            }"#}
        )
    }

    #[test]
    fn parses_cookie_parameter() {
        let value = json!({
            "name": "session_id",
            "in": "cookie"
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "session_id",
              "in": "cookie"
            }"#}
        )
    }

    #[test]
    fn parses_header_parameter() {
        let value = json!({
            "name": "X-Request-ID",
            "in": "header"
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "X-Request-ID",
              "in": "header"
            }"#}
        )
    }

    #[test]
    fn parses_query_parameter_with_allow_empty_value() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "allowEmptyValue": true
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "allowEmptyValue": true
            }"#}
        )
    }

    #[test]
    fn parses_query_parameter_with_allow_reserved() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "allowReserved": true
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "allowReserved": true
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_schema() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "schema": {
                "type": "number"
            }
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "schema": {
                "type": "number",
                "doctave_metadata": {
                  "field_name": "offset"
                }
              }
            }"#}
        )
    }

    #[test]
    fn overrides_schema_description() {
        let value = json!({
            "name": "offset",
            "description": "Overridden description",
            "in": "query",
            "schema": {
                "type": "number"
            }
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "schema": {
                "description": "Overridden description",
                "type": "number",
                "doctave_metadata": {
                  "field_name": "offset"
                }
              },
              "description": "Overridden description"
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_content() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "content": {
                "application/json": {
                    "schema": {
                        "type": "number"
                    }
                }
            }
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "content": {
                "application/json": {
                  "schema": {
                    "type": "number"
                  }
                }
              }
            }"#}
        )
    }

    #[test]
    fn parses_parameter_prioritizes_schema_over_content() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "schema": {
                "type": "number"
            },
            "content": {
                "application/json": {
                    "schema": {
                        "type": "number"
                    }
                }
            }
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "schema": {
                "type": "number",
                "doctave_metadata": {
                  "field_name": "offset"
                }
              }
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_example() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "example": 10
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "example": 10
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_examples() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "examples": {
                "example1": {
                    "value": 10
                }
            }
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "examples": {
                "example1": {
                  "value": 10
                }
              }
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_required() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "required": false
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "required": false
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_description() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "description": "The offset of the first item to return"
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "description": "The offset of the first item to return"
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_deprecated() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "deprecated": true
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "deprecated": true
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_explode() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "explode": true
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "explode": true
            }"#}
        )
    }

    #[test]
    fn parses_parameter_with_style() {
        let value = json!({
            "name": "offset",
            "in": "query",
            "style": "form"
        });
        let parameter =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "style": "form"
            }"#}
        )
    }

    #[test]
    fn fails_on_missing_name() {
        let value = json!({
            "in": "query",
            "schema": {
                "type": "string"
            }
        });
        let err =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(err.to_string(), r#"Parameter object should contain a name"#)
    }

    #[test]
    fn fails_on_missing_in() {
        let value = json!({
            "name": "test",
            "schema": {
                "type": "string"
            }
        });
        let err =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Parameter object should contain an 'in' field"#
        )
    }

    #[test]
    fn fails_on_invalid_in() {
        let value = json!({
            "name": "test",
            "in": "invalid",
            "schema": {
                "type": "string"
            }
        });
        let err =
            Parameter::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(err.to_string(), r#"Invalid 'in' value: invalid"#)
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/parameters/offset"
        });

        let ref_cache = json!({
          "parameters": {
            "offset": {
              "name": "offset",
              "in": "query",
              "schema": {
                "type": "number"
              }
            }
          }
        });

        let mut context = ParserContext::default();

        context.ref_cache.with_value(ref_cache);

        let parameter = Parameter::try_parse(value, &context, &mut Set::new()).unwrap();

        assert_str_eq!(
            parameter.pretty_print(),
            indoc! {r#"
            {
              "name": "offset",
              "in": "query",
              "schema": {
                "type": "number",
                "doctave_metadata": {
                  "field_name": "offset"
                }
              }
            }"#}
        );
    }
}
