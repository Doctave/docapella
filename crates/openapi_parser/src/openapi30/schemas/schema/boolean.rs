use crate::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct BooleanSchema {
    pub r#enum: Vec<bool>,
}

impl BooleanSchema {
    pub fn is_empty(&self) -> bool {
        self.r#enum.is_empty()
    }

    pub fn exclude(&mut self, other: &mut Self) {
        let common_enum: Vec<bool> = self
            .r#enum
            .iter()
            .filter(|value| other.r#enum.contains(value))
            .copied()
            .collect();

        self.r#enum.retain(|value| !common_enum.contains(value));
        other.r#enum.retain(|value| !common_enum.contains(value));
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.r#enum.extend(other.r#enum);

        self
    }
}

impl From<Value> for BooleanSchema {
    fn from(mut value: Value) -> Self {
        Self {
            r#enum: value
                .take("enum")
                .and_then(Value::take_array)
                .map(|arr| arr.into_iter().filter_map(Value::take_bool).collect())
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::Set;
    use crate::{
        json,
        openapi30::{parser::ParserContext, schemas::schema::Schema},
    };

    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_as_boolean() {
        let value = json!({
            "type": "boolean",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "boolean"
            }"# }
        )
    }

    #[test]
    fn parses_enum() {
        let value = json!({
            "type": "boolean",
            "enum": [true],
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "boolean",
              "enum": [
                true
              ]
            }"# }
        );
    }

    #[test]
    fn parses_default() {
        let value = json!({
            "type": "boolean",
            "default": true,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "boolean",
              "default": true
            }"# }
        );
    }

    #[test]
    fn parses_example() {
        let value = json!({
            "type": "boolean",
            "example": true,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "boolean",
              "example": true
            }"# }
        );
    }

    mod all_of {
        use super::*;

        #[test]
        fn merges_enum() {
            let value = json!({
              "type": "boolean",
              "allOf": [
                {
                  "type": "boolean",
                  "enum": [true]
                },
                {
                  "type": "boolean",
                  "enum": [false]
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "boolean",
                  "enum": [
                    true,
                    false
                  ]
                }"# }
            );
        }

        #[test]
        fn merges_default() {
            let value = json!({
              "type": "boolean",
              "allOf": [
                {
                  "type": "boolean",
                  "default": true
                },
                {
                  "type": "boolean",
                  "default": false
                }
              ]
            });

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! {r#"
                {
                  "type": "boolean",
                  "default": false
                }"# }
            );
        }

        mod not {
            use super::*;

            #[test]
            fn excludes_enum() {
                let value = json!({
                  "type": "boolean",
                  "allOf": [
                    {
                      "type": "boolean",
                      "enum": [true]
                    },
                    {
                      "not": {
                        "type": "boolean",
                        "enum": [true]
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
                      "type": "boolean"
                    }"# }
                );
            }

            #[test]
            fn excludes_default() {
                let value = json!({
                  "type": "boolean",
                  "allOf": [
                    {
                      "type": "boolean",
                      "default": true
                    },
                    {
                      "not": {
                        "type": "boolean",
                        "default": true
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
                      "type": "boolean"
                    }"# }
                );
            }
        }
    }
}
