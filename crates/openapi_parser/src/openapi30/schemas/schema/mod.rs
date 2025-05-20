pub mod any_of;
pub mod array;
pub mod boolean;
pub mod integer;
pub mod number;
pub mod object;
pub mod one_of;
pub mod property;
pub mod string;

use crate::openapi30::parser;
use crate::{openapi30::parser::ParserContext, String};

use crate::{Set, Value};

use crate::Map;
use integer::IntegerSchema;
use thiserror::Error;

use any_of::AnyOfSchema;
use array::ArraySchema;
use boolean::BooleanSchema;
use number::NumberSchema;
use object::ObjectSchema;
use one_of::OneOfSchema;
use string::StringSchema;

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid"#)]
    Invalid,
    #[error(r#"Invalid merge"#)]
    InvalidMerge,
    #[error(r#"Could not resolve reference '{0}'"#)]
    ReferenceNotFound(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum SchemaKind {
    String(StringSchema),
    Number(NumberSchema),
    Integer(IntegerSchema),
    Object(ObjectSchema),
    Array(ArraySchema),
    Boolean(BooleanSchema),
    OneOf(OneOfSchema),
    AnyOf(AnyOfSchema),
    Unknown,
}

impl SchemaKind {
    #[cfg(test)]
    pub(crate) fn as_object(&self) -> Option<&ObjectSchema> {
        match &self {
            SchemaKind::Object(obj) => Some(obj),
            _ => None,
        }
    }

    #[cfg(test)]
    #[allow(unused)]
    pub(crate) fn as_number(&self) -> Option<&NumberSchema> {
        match &self {
            SchemaKind::Number(obj) => Some(obj),
            _ => None,
        }
    }

    pub(crate) fn is_empty(&self) -> bool {
        match self {
            SchemaKind::Object(o) => o.is_empty(),
            SchemaKind::Number(n) => n.is_empty(),
            SchemaKind::Integer(n) => n.is_empty(),
            SchemaKind::Array(a) => a.is_empty(),
            SchemaKind::String(s) => s.is_empty(),
            SchemaKind::Boolean(b) => b.is_empty(),
            SchemaKind::OneOf(oo) => oo.is_empty(),
            SchemaKind::AnyOf(aa) => aa.is_empty(),
            SchemaKind::Unknown => true,
        }
    }

    pub(crate) fn exclude(&mut self, other: &mut Self) {
        match (self, other) {
            (SchemaKind::Object(a), SchemaKind::Object(b)) => a.exclude(b),
            (SchemaKind::Number(a), SchemaKind::Number(b)) => a.exclude(b),
            (SchemaKind::Integer(a), SchemaKind::Integer(b)) => a.exclude(b),
            (SchemaKind::Array(a), SchemaKind::Array(b)) => a.exclude(b),
            (SchemaKind::String(a), SchemaKind::String(b)) => a.exclude(b),
            (SchemaKind::Boolean(a), SchemaKind::Boolean(b)) => a.exclude(b),
            (_, _) => {}
        }
    }

    pub(crate) fn merge(self, other: Self) -> Self {
        match (self, other) {
            (SchemaKind::Object(a), SchemaKind::Object(b)) => SchemaKind::Object(a.merge(b)),
            (SchemaKind::Number(a), SchemaKind::Number(b)) => SchemaKind::Number(a.merge(b)),
            (SchemaKind::Integer(a), SchemaKind::Integer(b)) => SchemaKind::Integer(a.merge(b)),
            (SchemaKind::Array(a), SchemaKind::Array(b)) => SchemaKind::Array(a.merge(b)),
            (SchemaKind::String(a), SchemaKind::String(b)) => SchemaKind::String(a.merge(b)),
            (SchemaKind::Boolean(a), SchemaKind::Boolean(b)) => SchemaKind::Boolean(a.merge(b)),
            (SchemaKind::OneOf(a), SchemaKind::OneOf(b)) => SchemaKind::OneOf(a.merge(b)),
            (SchemaKind::AnyOf(a), SchemaKind::AnyOf(b)) => SchemaKind::AnyOf(a.merge(b)),
            (SchemaKind::Unknown, b) => b,
            (a, SchemaKind::Unknown) => a,
            (a, _) => a,
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Metadata {
    pub field_name: Option<String>,
    pub component_name: Option<String>,
    pub recursive: Option<String>,
    pub title: Option<String>,
}

impl From<Metadata> for Value {
    fn from(value: Metadata) -> Self {
        let mut val = Map::new();

        if let Some(name) = value.field_name {
            val.insert("field_name".into(), name.into());
        }

        if let Some(name) = value.component_name {
            val.insert("component_name".into(), name.into());
        }

        if let Some(rec) = value.recursive {
            val.insert("recursive".into(), rec.into());
        }

        if let Some(title) = value.title {
            val.insert("title".into(), title.into());
        }

        Value::Object(val)
    }
}

impl From<Value> for Metadata {
    fn from(mut val: Value) -> Self {
        Self {
            field_name: None,
            component_name: val.take("component_name").and_then(Value::take_string),
            recursive: val.take("recursive").and_then(Value::take_string),
            title: val.take("title").and_then(Value::take_string),
        }
    }
}

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Overrides {
    pub description: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Schema {
    pub metadata: Metadata,
    pub kind: SchemaKind,
    pub title: Option<String>,
    pub description: Option<String>,
    pub nullable: Option<bool>,
    pub not: Option<Box<Schema>>,
    pub example: Option<Value>,
    pub default: Option<Value>,
    pub deprecated: Option<bool>,
}

impl Schema {
    #[cfg(test)]
    pub(crate) fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    #[cfg(test)]
    pub fn child_schemas(&self) -> Vec<&Schema> {
        match &self.kind {
            SchemaKind::Object(o) => o
                .properties
                .values()
                .map(|s| &s.as_ref().schema)
                .collect::<Vec<_>>(),
            SchemaKind::Array(a) => a
                .items
                .as_ref()
                .map(|s| vec![s.as_ref()])
                .unwrap_or_default(),
            SchemaKind::OneOf(o) => o.one_of.iter().collect(),
            SchemaKind::AnyOf(a) => a.any_of.iter().collect(),
            _ => vec![],
        }
    }

    fn is_empty(&self) -> bool {
        self.kind.is_empty() && self.default.is_none() && self.example.is_none()
    }

    fn merge(mut self, other: Self) -> Self {
        self.kind = match (&self.kind, &other.kind) {
            (SchemaKind::OneOf(_), SchemaKind::OneOf(_)) => self.kind.merge(other.kind),
            (SchemaKind::AnyOf(_), SchemaKind::AnyOf(_)) => self.kind.merge(other.kind),
            (SchemaKind::AnyOf(_), SchemaKind::OneOf(_)) => self.kind,
            (SchemaKind::OneOf(_), SchemaKind::AnyOf(_)) => self.kind,
            (SchemaKind::AnyOf(a), _) => {
                let merged_any_of = a
                    .any_of
                    .clone()
                    .into_iter()
                    .map(|a| a.merge(other.clone()))
                    .collect::<Vec<_>>();

                SchemaKind::AnyOf(AnyOfSchema {
                    any_of: merged_any_of,
                })
            }
            (_, SchemaKind::AnyOf(a)) => {
                let merged_any_of = a
                    .any_of
                    .clone()
                    .into_iter()
                    .map(|a| a.merge(self.clone()))
                    .collect::<Vec<_>>();

                SchemaKind::AnyOf(AnyOfSchema {
                    any_of: merged_any_of,
                })
            }
            (SchemaKind::OneOf(o), _) => {
                let merged_one_of = o
                    .one_of
                    .clone()
                    .into_iter()
                    .map(|o| o.merge(other.clone()))
                    .collect::<Vec<_>>();

                SchemaKind::OneOf(OneOfSchema {
                    one_of: merged_one_of,
                })
            }
            (_, SchemaKind::OneOf(o)) => {
                let merged_one_of = o
                    .one_of
                    .clone()
                    .into_iter()
                    .map(|o| o.merge(self.clone()))
                    .collect::<Vec<_>>();

                SchemaKind::OneOf(OneOfSchema {
                    one_of: merged_one_of,
                })
            }
            _ => self.kind.merge(other.kind),
        };

        self.title = other.title.or(self.title);
        self.description = other.description.or(self.description);
        self.nullable = other.nullable.or(self.nullable);
        self.default = other.default.or(self.default);
        self.example = other.example.or(self.example);
        self.deprecated = other.deprecated.or(self.deprecated);

        match (self.not, other.not) {
            (Some(a), Some(b)) => {
                self.not = Some(Box::new(a.merge(*b)));
            }
            (a, b) => {
                self.not = b.or(a);
            }
        }

        self
    }

    fn exclude(&mut self) {
        if let Some(mut not) = self.not.take() {
            if let SchemaKind::OneOf(o) = &mut self.kind {
                for s in o.one_of.iter_mut() {
                    s.not = Some(not.clone());
                    s.exclude();
                }
            } else if let SchemaKind::AnyOf(a) = &mut self.kind {
                for s in a.any_of.iter_mut() {
                    s.not = Some(not.clone());
                    s.exclude();
                }
            } else {
                self.kind.exclude(&mut not.kind);
            }

            if self.default == not.default {
                self.default = None;
                not.default = None;
            }

            if self.example == not.example {
                self.example = None;
                not.example = None;
            }

            if not.is_empty() {
                self.not = None;
            } else {
                self.not = Some(not);
            }
        }
    }

    pub fn try_parse(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
        overrides: Option<Overrides>,
    ) -> parser::Result<Self> {
        let mut visited_refs = visited_refs.clone();

        let mut val = if let Some(all_of) = value.take("allOf").and_then(Value::take_array) {
            // There's a bit of juggling here to ensure we
            // 1. Compute merged allOf schema
            // 2. !important! we merge root schema to the merged schema (in order to override merged with the root)
            // 3. Use the root schema's metadata
            let mut merged_all_of: Option<Schema> = None;

            for val in all_of {
                let schema = Schema::try_parse(val, ctx, &mut visited_refs, None)?;

                if let Some(merged) = merged_all_of {
                    merged_all_of = Some(merged.merge(schema));
                } else {
                    merged_all_of = Some(schema);
                }
            }

            let mut root_all_of = Schema::try_parse(value, ctx, &mut visited_refs, overrides)?;

            if let Some(mut merged) = merged_all_of {
                let met = root_all_of.metadata;
                root_all_of.metadata = Metadata::default();

                merged.metadata = met;
                merged.merge(root_all_of)
            } else {
                root_all_of
            }
        } else if let Some(one_of) = value.take("oneOf").and_then(Value::take_array) {
            let mut resolved_one_of = vec![];

            for v in one_of {
                if let Ok(schema) = Schema::try_parse(v, ctx, &mut visited_refs, None) {
                    resolved_one_of.push(schema);
                }
            }

            Self {
                kind: SchemaKind::OneOf(OneOfSchema {
                    one_of: resolved_one_of,
                }),
                title: None,
                description: overrides
                    .and_then(|o| o.description)
                    .or(value.take("description").and_then(Value::take_string)),
                nullable: value.take("nullable").and_then(Value::take_bool),
                deprecated: value.take("deprecated").and_then(Value::take_bool),
                not: None,
                metadata: value
                    .take("doctave_metadata")
                    .map(Metadata::from)
                    .unwrap_or_default(),
                example: None,
                default: None,
            }
        } else if let Some(any_of) = value.take("anyOf").and_then(Value::take_array) {
            let mut resolved_any_of = vec![];

            for v in any_of {
                if let Ok(schema) = Schema::try_parse(v, ctx, &mut visited_refs, None) {
                    resolved_any_of.push(schema);
                }
            }

            Self {
                kind: SchemaKind::AnyOf(AnyOfSchema {
                    any_of: resolved_any_of,
                }),
                title: None,
                description: overrides
                    .and_then(|o| o.description)
                    .or(value.take("description").and_then(Value::take_string)),
                nullable: value.take("nullable").and_then(Value::take_bool),
                deprecated: value.take("deprecated").and_then(Value::take_bool),
                not: None,
                metadata: value
                    .take("doctave_metadata")
                    .map(Metadata::from)
                    .unwrap_or_default(),
                example: None,
                default: None,
            }
        } else if let Some(ref_path) = value.take("$ref").and_then(Value::take_string) {
            let value = ctx
                .get(&ref_path, &mut visited_refs)
                .ok_or(Error::ReferenceNotFound(ref_path))?;

            Schema::try_parse(value, ctx, &mut visited_refs, None)?
        } else {
            let not = if let Some(n) = value.take("not") {
                Some(Box::new(Schema::try_parse(
                    n,
                    ctx,
                    &mut visited_refs,
                    None,
                )?))
            } else {
                None
            };

            let title = value.take("title").and_then(Value::take_string);
            let mut metadata = value
                .take("doctave_metadata")
                .map(Metadata::from)
                .unwrap_or_default();

            metadata.title = title.clone();

            let mut description = overrides
                .and_then(|o| o.description)
                .or(value.take("description").and_then(Value::take_string));

            if let Some(guess) = &metadata.recursive {
                description = Some(format!("<p class='type-name'>reference ({guess})</p> <span class='recursive'>recursive</span>").into());
            }

            let nullable = value.take("nullable").and_then(Value::take_bool);

            let example = value.take("example");
            let default = value.take("default");

            let deprecated = value.take("deprecated").and_then(Value::take_bool);

            let kind = if let Some(t) = value.get("type").and_then(Value::as_str) {
                match t {
                    "boolean" => SchemaKind::Boolean(BooleanSchema::from(value)),
                    "object" => {
                        SchemaKind::Object(ObjectSchema::from_value(value, ctx, &mut visited_refs)?)
                    }
                    "array" => {
                        SchemaKind::Array(ArraySchema::from_value(value, ctx, &mut visited_refs)?)
                    }
                    "number" => SchemaKind::Number(NumberSchema::from(value)),
                    "integer" => SchemaKind::Integer(IntegerSchema::from(value)),
                    "string" => SchemaKind::String(StringSchema::from(value)),
                    _ => SchemaKind::Unknown,
                }
            } else if value.get("properties").is_some() {
                SchemaKind::Object(ObjectSchema::from_value(value, ctx, &mut visited_refs)?)
            } else if value.get("items").is_some() {
                SchemaKind::Array(ArraySchema::from_value(value, ctx, &mut visited_refs)?)
            } else {
                SchemaKind::Unknown
            };

            Self {
                title,
                not,
                kind,
                description,
                nullable,
                metadata,
                example,
                default,
                deprecated,
            }
        };

        val.exclude();

        Ok(val)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::json;

    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_title_and_adds_it_as_metadata() {
        let value = json!({
            "type": "object",
            "title": "Test",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "title": "Test",
              "type": "object",
              "doctave_metadata": {
                "title": "Test"
              }
            }"#}
        )
    }

    #[test]
    fn parses_description() {
        let value = json!({
            "type": "object",
            "description": "Test",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "description": "Test",
              "type": "object"
            }"#}
        );
    }

    #[test]
    fn parses_nullable() {
        let value = json!({
            "type": "object",
            "nullable": true,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "nullable": true,
              "type": "object"
            }"#}
        );
    }

    #[test]
    fn parses_not() {
        let value = json!({
            "type": "object",
            "not": {
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string"
                    }
                }
            },
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "not": {
                "type": "object",
                "properties": {
                  "name": {
                    "type": "string",
                    "doctave_metadata": {
                      "field_name": "name"
                    }
                  }
                }
              },
              "type": "object"
            }"#}
        );
    }

    #[test]
    fn parses_with_ref_cache_value() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
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

        let account_value = json!({
              "type": "object",
              "properties": {
                "id": {
                  "type": "number"
                },
                "name": {
                  "type": "string"
                },
                "user": {
                  "$ref": "#/components/schemas/User"
                }
              }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let account_schema = Schema::try_parse(account_value, &ctx, &mut Set::new(), None).unwrap();

        let user = &**account_schema
            .kind
            .as_object()
            .unwrap()
            .properties
            .get("user")
            .unwrap();

        assert_str_eq!(
            user.clone().pretty_print(),
            indoc! { r#"
          {
            "title": "Some User",
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
            },
            "doctave_metadata": {
              "field_name": "user",
              "component_name": "User",
              "title": "Some User"
            }
          }"# }
        );
    }

    #[test]
    fn doesnt_merge_metadata() {
        let components_value = json!({
          "schemas": {
            "User": {
              "type": "object",
              "allOf": [
                {
                  "$ref": "#/components/schemas/Foo"
                }
              ],
              "properties": {
                "age": {
                  "type": "number"
                },
              }
            },
            "Foo": {
              "type": "object",
              "properties": {
                "time": {
                  "type": "number"
                },
              }
            }
          }
        });

        let account_value = json!(
          {
            "oneOf": [
              {
                "$ref": "#/components/schemas/User"
              }
            ]
          }
        );

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let account_schema = Schema::try_parse(account_value, &ctx, &mut Set::new(), None).unwrap();

        assert_str_eq!(
            account_schema.pretty_print(),
            indoc! {r#"
            {
              "oneOf": [
                {
                  "type": "object",
                  "properties": {
                    "time": {
                      "type": "number",
                      "doctave_metadata": {
                        "field_name": "time"
                      }
                    },
                    "age": {
                      "type": "number",
                      "doctave_metadata": {
                        "field_name": "age"
                      }
                    }
                  },
                  "doctave_metadata": {
                    "component_name": "User"
                  }
                }
              ]
            }"#}
        );
    }

    #[test]
    fn all_of_with_ref() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "age": {
                  "type": "number"
                },
                "team_name": {
                  "type": "string"
                }
              }
            }
          }
        });

        let account_value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number"
                    },
                    "name": {
                      "type": "string"
                    },
                  }
                },
                {
                  "$ref": "#/components/schemas/User"
                }
              ]
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let account_schema = Schema::try_parse(account_value, &ctx, &mut Set::new(), None).unwrap();

        assert_str_eq!(
            account_schema.pretty_print(),
            indoc! {r#"
            {
              "title": "Some User",
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
                },
                "age": {
                  "type": "number",
                  "doctave_metadata": {
                    "field_name": "age"
                  }
                },
                "team_name": {
                  "type": "string",
                  "doctave_metadata": {
                    "field_name": "team_name"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn all_of_with_not() {
        let value = json!({
            "type": "object",
            "allOf": [
                {
                    "type": "object",
                    "properties": {
                        "id": {
                            "type": "number"
                        },
                        "name": {
                            "type": "string"
                        },
                    }
                },
                {
                    "not": {
                        "type": "object",
                        "properties": {
                            "age": {
                                "type": "number"
                            },
                            "team_name": {
                                "type": "string"
                            }
                        }
                    }
                }
            ]
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "not": {
                "type": "object",
                "properties": {
                  "age": {
                    "type": "number",
                    "doctave_metadata": {
                      "field_name": "age"
                    }
                  },
                  "team_name": {
                    "type": "string",
                    "doctave_metadata": {
                      "field_name": "team_name"
                    }
                  }
                }
              },
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
            }"#}
        );
    }

    #[test]
    fn parses_unknown_kind() {
        let value = json!({
            "title": "Test",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "title": "Test",
              "doctave_metadata": {
                "title": "Test"
              }
            }"#}
        );
    }

    #[test]
    fn prioritizes_first_kind_when_both_are_known_in_merge_if_incompatible() {
        let value = json!({
            "allOf": [
                {
                    "type": "object",
                    "properties": {
                        "age": {
                            "type": "number"
                        },
                        "team_name": {
                            "type": "string"
                        }
                    }
                },
                {
                    "type": "number",
                }
            ]
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "properties": {
                "age": {
                  "type": "number",
                  "doctave_metadata": {
                    "field_name": "age"
                  }
                },
                "team_name": {
                  "type": "string",
                  "doctave_metadata": {
                    "field_name": "team_name"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn prioritizes_known_kind_in_merge() {
        let value = json!({
            "allOf": [
                {
                    "properties": {
                        "age": {
                            "type": "number"
                        },
                        "team_name": {
                            "type": "string"
                        }
                    }
                },
                {
                    "type": "object",
                    "properties": {
                        "age": {
                            "type": "number"
                        },
                        "team_name": {
                            "type": "string"
                        }
                    }
                }
            ]
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "properties": {
                "age": {
                  "type": "number",
                  "doctave_metadata": {
                    "field_name": "age"
                  }
                },
                "team_name": {
                  "type": "string",
                  "doctave_metadata": {
                    "field_name": "team_name"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn merges_self_with_allof() {
        let value = json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "number"
                },
                "name": {
                    "type": "string"
                },
            },
            "allOf": [
                {
                    "type": "object",
                    "properties": {
                        "age": {
                            "type": "number"
                        },
                        "team_name": {
                            "type": "string"
                        }
                    }
                }
            ]
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "properties": {
                "age": {
                  "type": "number",
                  "doctave_metadata": {
                    "field_name": "age"
                  }
                },
                "team_name": {
                  "type": "string",
                  "doctave_metadata": {
                    "field_name": "team_name"
                  }
                },
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
            }"#}
        );
    }

    #[test]
    fn merges_self_with_allof_prioritizes_self() {
        let value = json!({
          "type": "object",
          "properties": {
            "id": {
              "type": "string"
            }
          },
          "allOf": [
            {
              "type": "object",
              "properties": {
                "id": {
                  "type": "number"
                }
              }
            }
          ]
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
        {
          "type": "object",
          "properties": {
            "id": {
              "type": "string",
              "doctave_metadata": {
                "field_name": "id"
              }
            }
          }
        }"# }
        );
    }

    #[test]
    fn parses_metadata() {
        let value = json!({
            "type": "string",
            "doctave_metadata": {
              "component_name": "test"
            }
        });

        let schema = Schema::try_parse(
            value.clone(),
            &ParserContext::default(),
            &mut Set::new(),
            None,
        )
        .unwrap();

        assert_eq!(Value::from(schema), value);
    }

    #[test]
    fn parses_one_of() {
        let value = json!({
            "oneOf": [
                {"type": "string"},
                {"type": "number"}
            ]
        });
        let one_of =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
          {
            "oneOf": [
              {
                "type": "string"
              },
              {
                "type": "number"
              }
            ]
          }"#}
        );
    }

    #[test]
    fn parses_one_of_from_ref_cache() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
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
        let value = json!({
            "oneOf": [
                {
                    "$ref": "#/components/schemas/User"
                },
                {"type": "number"}
            ]
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();

        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
          {
            "oneOf": [
              {
                "title": "Some User",
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
                },
                "doctave_metadata": {
                  "component_name": "User",
                  "title": "Some User"
                }
              },
              {
                "type": "number"
              }
            ]
          }"#}
        );
    }

    #[test]
    fn parses_any_of() {
        let value = json!({
            "anyOf": [
                {"type": "string"},
                {"type": "number"}
            ]
        });
        let one_of =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
          {
            "anyOf": [
              {
                "type": "string"
              },
              {
                "type": "number"
              }
            ]
          }"#}
        );
    }

    #[test]
    fn parses_any_of_from_ref_cache() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
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
        let value = json!({
            "anyOf": [
                {
                    "$ref": "#/components/schemas/User"
                },
                {"type": "number"}
            ]
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
          {
            "anyOf": [
              {
                "title": "Some User",
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
                },
                "doctave_metadata": {
                  "component_name": "User",
                  "title": "Some User"
                }
              },
              {
                "type": "number"
              }
            ]
          }"#}
        );
    }

    #[test]
    fn handles_recursive_gracefully() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "id": {
                  "type": "number"
                },
                "team": {
                  "$ref": "#/components/schemas/Team"
                }
              }
            },
            "Team": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "users": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/User"
                  }
                },
                "name": {
                  "type": "string"
                }
              }
            }
          }
        });

        let value = json!({
              "type": "object",
              "properties": {
                "user": {
                  "$ref": "#/components/schemas/User"
                }
              }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "properties": {
                "user": {
                  "title": "Some User",
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number",
                      "doctave_metadata": {
                        "field_name": "id"
                      }
                    },
                    "team": {
                      "title": "Some User",
                      "type": "object",
                      "properties": {
                        "name": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "name"
                          }
                        },
                        "users": {
                          "type": "array",
                          "items": {
                            "description": "<p class='type-name'>reference (User)</p> <span class='recursive'>recursive</span>",
                            "doctave_metadata": {
                              "recursive": "User"
                            }
                          },
                          "doctave_metadata": {
                            "field_name": "users"
                          }
                        }
                      },
                      "doctave_metadata": {
                        "field_name": "team",
                        "component_name": "Team",
                        "title": "Some User"
                      }
                    }
                  },
                  "doctave_metadata": {
                    "field_name": "user",
                    "component_name": "User",
                    "title": "Some User"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn doesnt_detect_sibling_refs_as_recursive() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "id": {
                  "type": "number"
                },
                "team": {
                  "$ref": "#/components/schemas/Team"
                },
                "team2": {
                  "$ref": "#/components/schemas/Team"
                }
              }
            },
            "Team": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "name": {
                  "type": "string"
                }
              }
            }
          }
        });

        let value = json!({
              "type": "object",
              "properties": {
                "user": {
                  "$ref": "#/components/schemas/User"
                }
              }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "properties": {
                "user": {
                  "title": "Some User",
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number",
                      "doctave_metadata": {
                        "field_name": "id"
                      }
                    },
                    "team": {
                      "title": "Some User",
                      "type": "object",
                      "properties": {
                        "name": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "name"
                          }
                        }
                      },
                      "doctave_metadata": {
                        "field_name": "team",
                        "component_name": "Team",
                        "title": "Some User"
                      }
                    },
                    "team2": {
                      "title": "Some User",
                      "type": "object",
                      "properties": {
                        "name": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "name"
                          }
                        }
                      },
                      "doctave_metadata": {
                        "field_name": "team2",
                        "component_name": "Team",
                        "title": "Some User"
                      }
                    }
                  },
                  "doctave_metadata": {
                    "field_name": "user",
                    "component_name": "User",
                    "title": "Some User"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn resolves_nested_references() {
        let components_value = json!({
          "schemas": {
            "User": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "id": {
                  "type": "number"
                },
                "team": {
                  "$ref": "#/components/schemas/Team"
                },
              }
            },
            "Team": {
              "$ref": "#/components/schemas/SubTeam1"
            },
            "SubTeam1": {
              "$ref": "#/components/schemas/SubTeam2"
            },
            "SubTeam2": {
              "title": "Some Team",
              "type": "object",
              "properties": {
                "name": {
                  "type": "string"
                }
              }
            }
          }
        });

        let value = json!({
              "type": "object",
              "properties": {
                "user": {
                  "$ref": "#/components/schemas/User"
                }
              }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
              {
                "type": "object",
                "properties": {
                  "user": {
                    "title": "Some User",
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "number",
                        "doctave_metadata": {
                          "field_name": "id"
                        }
                      },
                      "team": {
                        "title": "Some Team",
                        "type": "object",
                        "properties": {
                          "name": {
                            "type": "string",
                            "doctave_metadata": {
                              "field_name": "name"
                            }
                          }
                        },
                        "doctave_metadata": {
                          "field_name": "team",
                          "component_name": "Team",
                          "title": "Some Team"
                        }
                      }
                    },
                    "doctave_metadata": {
                      "field_name": "user",
                      "component_name": "User",
                      "title": "Some User"
                    }
                  }
                }
              }"#}
        );
    }

    #[test]
    fn does_not_overwrite_any_or_one_of_child_descriptions() {
        let value = json!({
            "description": "Parent description",
            "oneOf": [
                {"type": "string", "description": "String description"},
                {"type": "number", "description": "Integer description"}
            ]
        });

        let ctx = ParserContext::default();
        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();

        assert_eq!(one_of.description, Some("Parent description".into()));
        assert_eq!(
            one_of.child_schemas()[0].description,
            Some("String description".into())
        );
        assert_eq!(
            one_of.child_schemas()[1].description,
            Some("Integer description".into())
        );
    }

    #[test]
    fn parses_referenced_one_ofs_correctly() {
        let components_value = json!({
          "schemas": {
            "Team": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "users": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/User"
                  }
                },
                "name": {
                  "type": "string"
                }
              }
            },
            "User": {
              "oneOf": [
                {
                  "title": "Some User",
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number"
                    },
                    "team": {
                      "$ref": "#/components/schemas/Team"
                    },
                  }
                },
                {
                  "title": "Some User2",
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number"
                    },
                    "team2": {
                      "$ref": "#/components/schemas/Team"
                    },
                  }
                }
              ]
            }
          }
        });

        let value = json!({
              "type": "object",
              "properties": {
                "user": {
                  "$ref": "#/components/schemas/User"
                }
              }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
              {
                "type": "object",
                "properties": {
                  "user": {
                    "oneOf": [
                      {
                        "title": "Some User",
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "number",
                            "doctave_metadata": {
                              "field_name": "id"
                            }
                          },
                          "team": {
                            "title": "Some User",
                            "type": "object",
                            "properties": {
                              "name": {
                                "type": "string",
                                "doctave_metadata": {
                                  "field_name": "name"
                                }
                              },
                              "users": {
                                "type": "array",
                                "items": {
                                  "description": "<p class='type-name'>reference (User)</p> <span class='recursive'>recursive</span>",
                                  "doctave_metadata": {
                                    "recursive": "User"
                                  }
                                },
                                "doctave_metadata": {
                                  "field_name": "users"
                                }
                              }
                            },
                            "doctave_metadata": {
                              "field_name": "team",
                              "component_name": "Team",
                              "title": "Some User"
                            }
                          }
                        },
                        "doctave_metadata": {
                          "title": "Some User"
                        }
                      },
                      {
                        "title": "Some User2",
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "number",
                            "doctave_metadata": {
                              "field_name": "id"
                            }
                          },
                          "team2": {
                            "title": "Some User",
                            "type": "object",
                            "properties": {
                              "name": {
                                "type": "string",
                                "doctave_metadata": {
                                  "field_name": "name"
                                }
                              },
                              "users": {
                                "type": "array",
                                "items": {
                                  "description": "<p class='type-name'>reference (User)</p> <span class='recursive'>recursive</span>",
                                  "doctave_metadata": {
                                    "recursive": "User"
                                  }
                                },
                                "doctave_metadata": {
                                  "field_name": "users"
                                }
                              }
                            },
                            "doctave_metadata": {
                              "field_name": "team2",
                              "component_name": "Team",
                              "title": "Some User"
                            }
                          }
                        },
                        "doctave_metadata": {
                          "title": "Some User2"
                        }
                      }
                    ],
                    "doctave_metadata": {
                      "field_name": "user",
                      "component_name": "User"
                    }
                  }
                }
              }"#}
        );
    }

    #[test]
    fn parses_referenced_any_ofs_correctly() {
        let components_value = json!({
          "schemas": {
            "Team": {
              "title": "Some User",
              "type": "object",
              "properties": {
                "users": {
                  "type": "array",
                  "items": {
                    "$ref": "#/components/schemas/User"
                  }
                },
                "name": {
                  "type": "string"
                }
              }
            },
            "User": {
              "anyOf": [
                {
                  "title": "Some User",
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number"
                    },
                    "team": {
                      "$ref": "#/components/schemas/Team"
                    },
                  }
                },
                {
                  "title": "Some User2",
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number"
                    },
                    "team2": {
                      "$ref": "#/components/schemas/Team"
                    },
                  }
                }
              ]
            }
          }
        });

        let value = json!({
              "type": "object",
              "properties": {
                "user": {
                  "$ref": "#/components/schemas/User"
                }
              }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(components_value);

        let one_of = Schema::try_parse(value, &ctx, &mut Set::new(), None).unwrap();
        assert_str_eq!(
            one_of.pretty_print(),
            indoc! {r#"
              {
                "type": "object",
                "properties": {
                  "user": {
                    "anyOf": [
                      {
                        "title": "Some User",
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "number",
                            "doctave_metadata": {
                              "field_name": "id"
                            }
                          },
                          "team": {
                            "title": "Some User",
                            "type": "object",
                            "properties": {
                              "name": {
                                "type": "string",
                                "doctave_metadata": {
                                  "field_name": "name"
                                }
                              },
                              "users": {
                                "type": "array",
                                "items": {
                                  "description": "<p class='type-name'>reference (User)</p> <span class='recursive'>recursive</span>",
                                  "doctave_metadata": {
                                    "recursive": "User"
                                  }
                                },
                                "doctave_metadata": {
                                  "field_name": "users"
                                }
                              }
                            },
                            "doctave_metadata": {
                              "field_name": "team",
                              "component_name": "Team",
                              "title": "Some User"
                            }
                          }
                        },
                        "doctave_metadata": {
                          "title": "Some User"
                        }
                      },
                      {
                        "title": "Some User2",
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "number",
                            "doctave_metadata": {
                              "field_name": "id"
                            }
                          },
                          "team2": {
                            "title": "Some User",
                            "type": "object",
                            "properties": {
                              "name": {
                                "type": "string",
                                "doctave_metadata": {
                                  "field_name": "name"
                                }
                              },
                              "users": {
                                "type": "array",
                                "items": {
                                  "description": "<p class='type-name'>reference (User)</p> <span class='recursive'>recursive</span>",
                                  "doctave_metadata": {
                                    "recursive": "User"
                                  }
                                },
                                "doctave_metadata": {
                                  "field_name": "users"
                                }
                              }
                            },
                            "doctave_metadata": {
                              "field_name": "team2",
                              "component_name": "Team",
                              "title": "Some User"
                            }
                          }
                        },
                        "doctave_metadata": {
                          "title": "Some User2"
                        }
                      }
                    ],
                    "doctave_metadata": {
                      "field_name": "user",
                      "component_name": "User"
                    }
                  }
                }
              }"#}
        );
    }
}
