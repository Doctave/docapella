use crate::{
    openapi30::parser::{self, ParserContext},
    Set, String, Value,
};

#[cfg(test)]
use serde_json::to_string_pretty;

use super::Schema;

#[derive(Debug, Clone, PartialEq)]
pub struct OneOfSchema {
    pub one_of: Vec<Schema>,
}

impl OneOfSchema {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.clone().into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn is_empty(&self) -> bool {
        self.one_of.is_empty()
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.one_of.extend(other.one_of);
        self.one_of.dedup();

        self
    }

    pub fn exclude(self, _other: &mut Self) -> Self {
        // Todo

        self
    }

    pub fn try_parse(
        value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let one_of = value
            .take_array()
            .unwrap_or_default()
            .into_iter()
            .map(|v| Schema::try_parse(v, ctx, visited_refs, None))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(OneOfSchema { one_of })
    }
}

#[cfg(test)]
mod test {
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use super::*;

    #[test]
    fn parses_one_of_schema() {
        let value = json!([
            {"type": "string"},
            {"type": "number"}
        ]);
        let one_of_schema =
            OneOfSchema::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            one_of_schema.pretty_print(),
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

    mod all_of {
        use super::*;

        #[test]
        fn merges_one_of_with_one_of() {
            let value = json!({
              "allOf": [
                {
                  "oneOf": [
                    {"type": "string"},
                    {"type": "number"}
                  ]
                },
                {
                  "oneOf": [
                    {"type": "boolean"},
                    {"type": "object"}
                  ]
                }
              ]
            });

            let all_of_schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                all_of_schema.pretty_print(),
                indoc! {r#"
                {
                  "oneOf": [
                    {
                      "type": "string"
                    },
                    {
                      "type": "number"
                    },
                    {
                      "type": "boolean"
                    },
                    {
                      "type": "object"
                    }
                  ]
                }"#}
            );
        }

        #[test]
        fn merges_one_of_with_object() {
            let value = json!({
              "allOf": [
                {
                  "oneOf": [
                    {
                      "type": "object",
                      "properties": {
                        "surname": {
                          "type": "string"
                        },
                      }
                    },
                    {
                      "type": "object",
                      "properties": {
                        "parent_name": {
                          "type": "string"
                        }
                      }
                    }
                  ]
                },
                {
                  "type": "object",
                  "properties": {
                    "name": {
                      "type": "string"
                    }
                  }
                }
              ]
            });

            let all_of_schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                all_of_schema.pretty_print(),
                indoc! {r#"
                {
                  "oneOf": [
                    {
                      "type": "object",
                      "properties": {
                        "surname": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "surname"
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
                    {
                      "type": "object",
                      "properties": {
                        "parent_name": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "parent_name"
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
                  ]
                }"#}
            );
        }
    }

    mod not {
        use super::*;

        #[test]
        #[ignore]
        fn excludes_identical_values() {
            let value = json!({
              "oneOf": [
                {"type": "string"},
                {"type": "number"}
              ],
              "not": {
                "oneOf": [
                  {"type": "number"}
                ]
              }
            });

            let mut one_of_schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            one_of_schema.exclude();

            assert_str_eq!(
                one_of_schema.pretty_print(),
                indoc! {r#"
                {
                  "oneOf": [
                    {"type": "string"}
                  ]
                }"#}
            );
        }
    }
}
