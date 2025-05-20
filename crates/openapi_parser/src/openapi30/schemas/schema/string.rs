use crate::{Number, String, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct StringSchema {
    pub pattern: Option<String>,
    pub r#enum: Vec<String>,
    pub max_length: Option<Number>,
    pub min_length: Option<Number>,
    pub format: Option<String>, // https://swagger.io/specification/v3/#data-type-format
}

impl StringSchema {
    pub fn is_empty(&self) -> bool {
        self.pattern.is_none()
            && self.r#enum.is_empty()
            && self.max_length.is_none()
            && self.min_length.is_none()
            && self.format.is_none()
    }

    pub fn exclude(&mut self, other: &mut Self) {
        if self.pattern == other.pattern {
            self.pattern = None;
            other.pattern = None;
        }

        let common_enum: Vec<String> = self
            .r#enum
            .iter()
            .filter(|value| other.r#enum.contains(value))
            .cloned()
            .collect();

        self.r#enum.retain(|value| !common_enum.contains(value));
        other.r#enum.retain(|value| !common_enum.contains(value));

        if self.max_length == other.max_length {
            self.max_length = None;
            other.max_length = None;
        }

        if self.min_length == other.min_length {
            self.min_length = None;
            other.min_length = None;
        }

        if self.format == other.format {
            self.format = None;
            other.format = None;
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.r#enum.extend(other.r#enum);

        self.pattern = other.pattern.or(self.pattern);
        self.format = other.format.or(self.format);

        match (&self.max_length, &other.max_length) {
            (Some(self_max), Some(other_max)) => {
                if self_max > other_max {
                    self.max_length = other.max_length;
                }
            }
            (None, Some(_)) => self.max_length = other.max_length,
            _ => {}
        }

        match (&self.min_length, &other.min_length) {
            (Some(self_min), Some(other_min)) => {
                if self_min < other_min {
                    self.min_length = other.min_length;
                }
            }
            (None, Some(_)) => self.min_length = other.min_length,
            _ => {}
        }

        self
    }
}

impl From<Value> for StringSchema {
    fn from(mut value: Value) -> Self {
        Self {
            pattern: value.take("pattern").and_then(Value::take_string),
            r#enum: value
                .take("enum")
                .and_then(Value::take_array)
                .map(|arr| arr.into_iter().filter_map(Value::take_string).collect())
                .unwrap_or_default(),
            max_length: value.take("maxLength").and_then(Value::take_number),
            min_length: value.take("minLength").and_then(Value::take_number),
            format: value.take("format").and_then(Value::take_string),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        json,
        openapi30::{parser::ParserContext, schemas::schema::Schema},
        Set,
    };

    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_as_string() {
        let value = json!({
            "type": "string",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string"
            }"# }
        );
    }

    #[test]
    fn parses_enum() {
        let value = json!({
            "type": "string",
            "enum": ["foo"],
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "enum": [
                "foo"
              ]
            }"# }
        )
    }

    #[test]
    fn parses_pattern() {
        let value = json!({
            "type": "string",
            "pattern": "^[a-z]+$",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "pattern": "^[a-z]+$"
            }"# }
        )
    }

    #[test]
    fn parses_max_length() {
        let value = json!({
            "type": "string",
            "maxLength": 10,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "maxLength": 10
            }"# }
        );
    }

    #[test]
    fn parses_min_length() {
        let value = json!({
            "type": "string",
            "minLength": 3,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "minLength": 3
            }"# }
        );
    }

    #[test]
    fn parses_format() {
        let value = json!({
            "type": "string",
            "format": "email",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "format": "email"
            }"# }
        );
    }

    #[test]
    fn parses_default() {
        let value = json!({
            "type": "string",
            "default": "foo",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "default": "foo"
            }"# }
        );
    }

    #[test]
    fn parses_example() {
        let value = json!({
            "type": "string",
            "example": "foo",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "string",
              "example": "foo"
            }"# }
        );
    }

    mod all_of {
        use super::*;

        #[test]
        fn merges_strings() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "maxLength": 10
                },
                {
                  "type": "string",
                  "minLength": 3
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "maxLength": 10,
                  "minLength": 3
                }"# }
            );
        }

        #[test]
        fn merges_formats() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "format": "email"
                },
                {
                  "type": "string",
                  "format": "uri"
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "format": "uri"
                }"# }
            );
        }

        #[test]
        fn merges_enums() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "enum": ["foo"]
                },
                {
                  "type": "string",
                  "enum": ["bar"]
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "enum": [
                    "foo",
                    "bar"
                  ]
                }"# }
            );
        }

        #[test]
        fn merges_patterns() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "pattern": "^[a-z]+$"
                },
                {
                  "type": "string",
                  "pattern": "^[0-9]+$"
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "pattern": "^[0-9]+$"
                }"# }
            );
        }

        #[test]
        fn merges_defaults() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "default": "foo"
                },
                {
                  "type": "string",
                  "default": "bar"
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "default": "bar"
                }"# }
            );
        }

        #[test]
        fn merges_min_length() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "minLength": 3
                },
                {
                  "type": "string",
                  "minLength": 5
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "minLength": 5
                }"# }
            );
        }

        #[test]
        fn merges_min_length_lower_end() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "minLength": 5
                },
                {
                  "type": "string",
                  "minLength": 3
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "minLength": 5
                }"# }
            );
        }

        #[test]
        fn merges_max_length() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "maxLength": 10
                },
                {
                  "type": "string",
                  "maxLength": 5
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "maxLength": 5
                }"# }
            );
        }

        #[test]
        fn merges_max_length_higher_end() {
            let value = json!({
              "type": "string",
              "allOf": [
                {
                  "type": "string",
                  "maxLength": 10
                },
                {
                  "type": "string",
                  "maxLength": 5
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "string",
                  "maxLength": 5
                }"# }
            );
        }

        mod not {
            use super::*;

            #[test]
            fn excludes_formats() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "format": "email"
                    },
                    {
                      "not": {
                        "type": "string",
                        "format": "email"
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }

            #[test]
            fn excludes_enums() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "enum": ["foo"]
                    },
                    {
                      "not": {
                        "type": "string",
                        "enum": ["foo"]
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }

            #[test]
            fn excludes_patterns() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "pattern": "^[a-z]+$"
                    },
                    {
                      "not": {
                        "type": "string",
                        "pattern": "^[a-z]+$"
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }

            #[test]
            fn excludes_defaults() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "default": "foo"
                    },
                    {
                      "not": {
                        "type": "string",
                        "default": "foo"
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }

            #[test]
            fn excludes_min_length() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "minLength": 3
                    },
                    {
                      "not": {
                        "type": "string",
                        "minLength": 3
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }

            #[test]
            fn excludes_max_length() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "maxLength": 5
                    },
                    {
                      "not": {
                        "type": "string",
                        "maxLength": 5
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }

            #[test]
            fn excludes_multiple() {
                let value = json!({
                  "type": "string",
                  "allOf": [
                    {
                      "type": "string",
                      "format": "email",
                      "enum": ["foo"],
                      "pattern": "^[a-z]+$",
                      "default": "foo",
                      "minLength": 3,
                      "maxLength": 5
                    },
                    {
                      "not": {
                        "type": "string",
                        "format": "email",
                        "enum": ["foo"],
                        "pattern": "^[a-z]+$",
                        "default": "foo",
                        "minLength": 3,
                        "maxLength": 5
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "string"
                    }"# }
                );
            }
        }
    }
}
