use crate::{Number, String, Value};

#[derive(Debug, Clone, PartialEq)]
pub struct NumberSchema {
    pub multiple_of: Option<Number>,
    pub maximum: Option<Number>,
    pub exclusive_maximum: Option<bool>,
    pub minimum: Option<Number>,
    pub exclusive_minimum: Option<bool>,
    pub r#enum: Vec<Number>,
    pub format: Option<String>, // https://swagger.io/specification/v3/#data-type-format
}

impl NumberSchema {
    pub fn is_empty(&self) -> bool {
        self.multiple_of.is_none()
            && self.maximum.is_none()
            && self.minimum.is_none()
            && self.r#enum.is_empty()
            && self.format.is_none()
            && self.exclusive_maximum.is_none()
            && self.exclusive_minimum.is_none()
    }

    pub fn exclude(&mut self, other: &mut Self) {
        let common_enum: Vec<Number> = self
            .r#enum
            .iter()
            .filter(|value| other.r#enum.contains(value))
            .cloned()
            .collect();

        self.r#enum.retain(|value| !common_enum.contains(value));
        other.r#enum.retain(|value| !common_enum.contains(value));

        if self.multiple_of == other.multiple_of {
            self.multiple_of = None;
            other.multiple_of = None;
        }

        if self.maximum == other.maximum {
            self.maximum = None;
            other.maximum = None;
        }

        if self.minimum == other.minimum {
            self.minimum = None;
            other.minimum = None;
        }

        if self.exclusive_maximum == other.exclusive_maximum {
            self.exclusive_maximum = None;
            other.exclusive_maximum = None;
        }

        if self.exclusive_minimum == other.exclusive_minimum {
            self.exclusive_minimum = None;
            other.exclusive_minimum = None;
        }

        if self.format == other.format {
            self.format = None;
            other.format = None;
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        match (&self.maximum, &other.maximum) {
            (Some(self_max), Some(other_max)) => {
                if self_max > other_max {
                    self.maximum = other.maximum;
                }
            }
            (None, Some(_)) => {
                self.maximum = other.maximum;
            }
            _ => {}
        }

        match (&self.minimum, &other.minimum) {
            (Some(self_min), Some(other_min)) => {
                if self_min < other_min {
                    self.minimum = other.minimum;
                }
            }
            (None, Some(_)) => {
                self.minimum = other.minimum;
            }
            _ => {}
        }

        self.exclusive_maximum = other.exclusive_maximum.or(self.exclusive_maximum);
        self.exclusive_minimum = other.exclusive_minimum.or(self.exclusive_minimum);
        self.format = other.format.or(self.format);

        self.r#enum.extend(other.r#enum);

        self.multiple_of = other.multiple_of.or(self.multiple_of);
        self
    }
}

impl From<Value> for NumberSchema {
    fn from(mut value: Value) -> Self {
        Self {
            multiple_of: value.take("multipleOf").and_then(Value::take_number),
            maximum: value.take("maximum").and_then(Value::take_number),
            exclusive_maximum: value.take("exclusiveMaximum").and_then(Value::take_bool),
            minimum: value.take("minimum").and_then(Value::take_number),
            exclusive_minimum: value.take("exclusiveMinimum").and_then(Value::take_bool),
            r#enum: value
                .take("enum")
                .and_then(Value::take_array)
                .map(|arr| arr.into_iter().filter_map(Value::take_number).collect())
                .unwrap_or_default(),
            format: value.take("format").and_then(Value::take_string),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{
        json,
        openapi30::{parser::ParserContext, schemas::schema::Schema},
        Set,
    };

    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_as_number() {
        let value = json!({
            "type": "number",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number"
            }"#
            }
        );
    }

    #[test]
    fn parses_multiple_of() {
        let value = json!({
            "type": "number",
            "multipleOf": 2
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "multipleOf": 2
            }"#
            }
        );
    }

    #[test]
    fn parses_maximum() {
        let value = json!({
            "type": "number",
            "maximum": 10
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "maximum": 10
            }"#
            }
        );
    }

    #[test]
    fn parses_minimum() {
        let value = json!({
            "type": "number",
            "minimum": 10
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "minimum": 10
            }"#
            }
        );
    }

    #[test]
    fn parses_exclusive_maximum() {
        let value = json!({
            "type": "number",
            "exclusiveMaximum": true
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "exclusiveMaximum": true
            }"#
            }
        );
    }

    #[test]
    fn parses_exclusive_minimum() {
        let value = json!({
            "type": "number",
            "exclusiveMinimum": true
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "exclusiveMinimum": true
            }"#
            }
        );
    }

    #[test]
    fn parses_enum() {
        let value = json!({
            "type": "number",
            "enum": [1, 2, 3]
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "enum": [
                1,
                2,
                3
              ]
            }"#
            }
        );
    }

    #[test]
    fn parses_format() {
        let value = json!({
            "type": "number",
            "format": "int32"
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "format": "int32"
            }"#
            }
        );
    }

    #[test]
    fn parses_example() {
        let value = json!({
            "type": "number",
            "example": 1
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! { r#"
            {
              "type": "number",
              "example": 1
            }"#
            }
        );
    }

    mod all_of {
        use super::*;

        #[test]
        fn merges_numbers() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "multipleOf": 2
                },
                {
                  "type": "number",
                  "maximum": 10
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "maximum": 10,
                  "multipleOf": 2
                }"#
                }
            );
        }

        #[test]
        fn merges_format() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "format": "double"
                },
                {
                  "type": "number",
                  "format": "float"
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "format": "float"
                }"#
                }
            );
        }

        #[test]
        fn merges_enum() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "enum": [1, 2]
                },
                {
                  "type": "number",
                  "enum": [3, 4]
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "enum": [
                    1,
                    2,
                    3,
                    4
                  ]
                }"#
                }
            );
        }

        #[test]
        fn merges_default() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "default": 1
                },
                {
                  "type": "number",
                  "default": 2
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "default": 2
                }"#
                }
            );
        }

        #[test]
        fn merges_maximum() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "maximum": 1
                },
                {
                  "type": "number",
                  "maximum": 2
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "maximum": 1
                }"#
                }
            );
        }

        #[test]
        fn merges_maximum_uses_upper_end_value() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "maximum": 3
                },
                {
                  "type": "number",
                  "maximum": 1,
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "maximum": 1
                }"#
                }
            );
        }

        #[test]
        fn merges_minimum() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "minimum": 1
                },
                {
                  "type": "number",
                  "minimum": 2
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "minimum": 2
                }"#
                }
            );
        }

        #[test]
        fn merges_minimum_uses_lower_end_value() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "minimum": 3
                },
                {
                  "type": "number",
                  "minimum": 1,
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "minimum": 3
                }"#
                }
            );
        }

        #[test]
        fn merges_exclusive_maximum() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "exclusiveMaximum": true
                },
                {
                  "type": "number",
                  "exclusiveMaximum": false
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "exclusiveMaximum": false
                }"#
                }
            );
        }

        #[test]
        fn merges_exclusive_minimum() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "exclusiveMinimum": true
                },
                {
                  "type": "number",
                  "exclusiveMinimum": false
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "exclusiveMinimum": false
                }"#
                }
            );
        }

        #[test]
        fn merges_multiple_of() {
            let value = json!({
              "type": "number",
              "allOf": [
                {
                  "type": "number",
                  "multipleOf": 1
                },
                {
                  "type": "number",
                  "multipleOf": 2
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
                  "type": "number",
                  "multipleOf": 2
                }"#
                }
            );
        }

        mod not {
            use super::*;

            #[test]
            fn excludes_enum() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "enum": [1, 2]
                    },
                    {
                      "not": {
                        "type": "number",
                        "enum": [2]
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number",
                      "enum": [
                        1
                      ]
                    }"#}
                );
            }

            #[test]
            fn excludes_multiple_of() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "multipleOf": 1
                    },
                    {
                      "not": {
                        "type": "number",
                        "multipleOf": 1
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_maximum() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "maximum": 1
                    },
                    {
                      "not": {
                        "type": "number",
                        "maximum": 1
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_minimum() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "minimum": 1
                    },
                    {
                      "not": {
                        "type": "number",
                        "minimum": 1
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_exclusive_maximum() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "exclusiveMaximum": true
                    },
                    {
                      "not": {
                        "type": "number",
                        "exclusiveMaximum": true
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_exclusive_minimum() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "exclusiveMinimum": true
                    },
                    {
                      "not": {
                        "type": "number",
                        "exclusiveMinimum": true
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_multiple_properties() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "maximum": 1,
                      "minimum": 1
                    },
                    {
                      "not": {
                        "type": "number",
                        "maximum": 1,
                        "minimum": 1
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_format() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "format": "float"
                    },
                    {
                      "not": {
                        "type": "number",
                        "format": "float"
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }

            #[test]
            fn excludes_default() {
                let value = json!({
                  "type": "number",
                  "allOf": [
                    {
                      "type": "number",
                      "default": 1
                    },
                    {
                      "not": {
                        "type": "number",
                        "default": 1
                      }
                    }
                  ]
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! { r#"
                    {
                      "type": "number"
                    }"#}
                );
            }
        }
    }
}
