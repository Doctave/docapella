use crate::openapi30::parser::{ParserContext, Result};
use crate::Set;
use crate::{Number, String, Value};

use super::Schema;

#[derive(Debug, Clone, PartialEq)]
pub struct ArraySchema {
    pub unique_items: Option<bool>,
    pub max_items: Option<Number>,
    pub min_items: Option<Number>,
    pub items: Option<Box<Schema>>, // referable, we should resolve these out
}

impl ArraySchema {
    pub fn is_empty(&self) -> bool {
        self.unique_items.is_none()
            && self.max_items.is_none()
            && self.min_items.is_none()
            && self.items.is_none()
    }

    pub fn exclude(&mut self, other: &mut Self) {
        if self.unique_items == other.unique_items {
            self.unique_items = None;
            other.unique_items = None;
        }

        if self.max_items == other.max_items {
            self.max_items = None;
            other.max_items = None;
        }

        if self.min_items == other.min_items {
            self.min_items = None;
            other.min_items = None;
        }

        if self.items == other.items {
            // If we have exact items exclusion, remove both
            self.items = None;
            other.items = None;
        } else if let Some(self_val) = &mut self.items {
            // Otherwise, do partial exclusion
            self_val.not = other.items.take();
            self_val.exclude();
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.unique_items = other.unique_items.or(self.unique_items);

        match (&self.max_items, &other.max_items) {
            (Some(self_max), Some(other_max)) if self_max > other_max => {
                self.max_items = other.max_items;
            }
            (None, Some(_)) => {
                self.max_items = other.max_items;
            }
            _ => {}
        }

        match (&self.min_items, &other.min_items) {
            (Some(self_min), Some(other_min)) if self_min < other_min => {
                self.min_items = other.min_items;
            }
            (None, Some(_)) => {
                self.min_items = other.min_items;
            }
            _ => {}
        }

        if let Some(items) = self.items {
            if let Some(other_items) = other.items {
                self.items = Some(Box::new(items.merge(*other_items)));
            } else {
                self.items = Some(items);
            }
        } else {
            self.items = other.items;
        }

        self
    }

    pub fn from_value(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> Result<Self> {
        let items = value
            .take("items")
            .map(|v| Schema::try_parse(v, ctx, visited_refs, None))
            .transpose()?
            .map(Box::new);

        Ok(Self {
            unique_items: value.take("uniqueItems").and_then(Value::take_bool),
            max_items: value.take("maxItems").and_then(Value::take_number),
            min_items: value.take("minItems").and_then(Value::take_number),
            items,
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
    fn parses_as_array() {
        let value = json!({
            "type": "array",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array"
            }"#}
        );
    }

    #[test]
    fn parses_unique_items() {
        let value = json!({
            "type": "array",
            "uniqueItems": true,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array",
              "uniqueItems": true
            }"#}
        );
    }

    #[test]
    fn parses_max_items() {
        let value = json!({
            "type": "array",
            "maxItems": 10,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array",
              "maxItems": 10
            }"#}
        );
    }

    #[test]
    fn parses_min_items() {
        let value = json!({
            "type": "array",
            "minItems": 1,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array",
              "minItems": 1
            }"#}
        );
    }

    #[test]
    fn parses_items() {
        let value = json!({
            "type": "array",
            "items": {
                "type": "string",
            },
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array",
              "items": {
                "type": "string"
              }
            }"#}
        );
    }

    #[test]
    fn parses_default() {
        let value = json!({
            "type": "array",
            "default": ["a", "b", "c"],
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array",
              "default": [
                "a",
                "b",
                "c"
              ]
            }"#}
        );
    }

    #[test]
    fn parses_example() {
        let value = json!({
            "type": "array",
            "example": ["a", "b", "c"],
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "array",
              "example": [
                "a",
                "b",
                "c"
              ]
            }"#}
        );
    }

    mod all_of {
        use super::*;

        #[test]
        fn merges_unique_items() {
            let value = json!({
              "type": "array",
              "allOf": [
                    {
                        "type": "array",
                        "uniqueItems": true,
                    },
                    {
                        "type": "array",
                        "uniqueItems": false,
                    },
                ],
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "uniqueItems": false
                }"#}
            );
        }

        #[test]
        fn merges_max_items() {
            let value = json!({
              "type": "array",
              "allOf": [
                    {
                        "type": "array",
                        "maxItems": 10,
                    },
                    {
                        "type": "array",
                        "maxItems": 20,
                    },
                ],
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "maxItems": 10
                }"#}
            );
        }

        #[test]
        fn merges_max_items_upper_end() {
            let value = json!({
              "type": "array",
              "allOf": [
                    {
                        "type": "array",
                        "maxItems": 10,
                    },
                    {
                        "type": "array",
                        "maxItems": 20,
                    },
                ],
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "maxItems": 10
                }"#}
            );
        }

        #[test]
        fn merges_min_items() {
            let value = json!({
              "type": "array",
              "allOf": [
                    {
                        "type": "array",
                        "minItems": 1,
                    },
                    {
                        "type": "array",
                        "minItems": 2,
                    },
                ],
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "minItems": 2
                }"#}
            );
        }

        #[test]
        fn merges_min_items_lower_end() {
            let value = json!({
              "type": "array",
              "allOf": [
                    {
                        "type": "array",
                        "minItems": 2,
                    },
                    {
                        "type": "array",
                        "minItems": 1,
                    },
                ],
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "minItems": 2
                }"#}
            );
        }

        #[test]
        fn merges_default() {
            let value = json!({
              "type": "array",
              "allOf": [
                    {
                        "type": "array",
                        "default": ["a", "b"],
                    },
                    {
                        "type": "array",
                        "default": ["c", "d"],
                    },
                ],
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "default": [
                    "c",
                    "d"
                  ]
                }"#}
            );
        }

        #[test]
        fn merges_items() {
            let value = json!({
              "type": "array",
              "allOf": [
                {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "id": {
                        "type": "number"
                      },
                      "name": {
                        "type": "string"
                      },
                    }
                  }
                },
                {
                  "type": "array",
                  "items": {
                    "type": "object",
                    "properties": {
                      "job": {
                        "type": "string"
                      },
                      "age": {
                        "type": "number"
                      },
                    }
                  }
                },
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "array",
                  "items": {
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
                      "job": {
                        "type": "string",
                        "doctave_metadata": {
                          "field_name": "job"
                        }
                      },
                      "age": {
                        "type": "number",
                        "doctave_metadata": {
                          "field_name": "age"
                        }
                      }
                    }
                  }
                }"#}
            );
        }

        mod not {
            use super::*;

            #[test]
            fn excludes_unique_items() {
                let value = json!({
                  "type": "array",
                  "allOf": [
                        {
                            "type": "array",
                            "uniqueItems": true,
                        },
                        {
                            "not": {
                                "type": "array",
                                "uniqueItems": true,
                            }
                        },
                    ],
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "array"
                    }"#}
                );
            }

            #[test]
            fn excludes_max_items() {
                let value = json!({
                  "type": "array",
                  "allOf": [
                        {
                            "type": "array",
                            "maxItems": 10,
                        },
                        {
                            "not": {
                                "type": "array",
                                "maxItems": 10,
                            }
                        },
                    ],
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "array"
                    }"#}
                );
            }

            #[test]
            fn excludes_min_items() {
                let value = json!({
                  "type": "array",
                  "allOf": [
                        {
                            "type": "array",
                            "minItems": 1,
                        },
                        {
                            "not": {
                                "type": "array",
                                "minItems": 1,
                            }
                        },
                    ],
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "array"
                    }"#}
                );
            }

            #[test]
            fn excludes_items() {
                let value = json!({
                  "type": "array",
                  "allOf": [
                        {
                            "type": "array",
                            "items": {
                                "type": "string",
                            },
                        },
                        {
                            "not": {
                                "type": "array",
                                "items": {
                                    "type": "string",
                                },
                            }
                        },
                    ],
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "array"
                    }"#}
                );
            }

            #[test]
            fn excludes_items_partial() {
                let value = json!({
                  "type": "array",
                  "allOf": [
                        {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "id": {
                                        "type": "number",
                                    },
                                    "name": {
                                        "type": "string",
                                    },
                                },
                            },
                        },
                        {
                            "not": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "id": {
                                            "type": "number",
                                        },
                                    },
                                },
                            }
                        },
                    ],
                });

                let schema =
                    Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None)
                        .unwrap();

                assert_str_eq!(
                    schema.pretty_print(),
                    indoc! {r#"
                    {
                      "type": "array",
                      "items": {
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
                    }"#}
                );
            }
        }
    }
}
