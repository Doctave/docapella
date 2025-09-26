use crate::{Map, Set};

use crate::{
    openapi30::parser::{ParserContext, Result},
    Number, String, Value,
};

use super::property::Property;
use super::Schema;

#[derive(Debug, Clone, PartialEq)]
pub struct ObjectSchema {
    pub max_properties: Option<Number>,
    pub min_properties: Option<Number>,
    pub properties: Map<String, Box<Property>>, // referable
    pub additional_properties: Option<Box<Schema>>, // referable
    pub required: Set<String>,
}

impl ObjectSchema {
    pub fn is_empty(&self) -> bool {
        self.max_properties.is_none()
            && self.min_properties.is_none()
            && self.properties.is_empty()
            && self.additional_properties.is_none()
            && self.required.is_empty()
    }

    pub fn exclude(&mut self, other: &mut Self) {
        // -------- `max_properties` exclusion
        if self.max_properties == other.max_properties {
            self.max_properties = None;
            other.max_properties = None;
        }

        // -------- `min_properties` exclusion
        if self.min_properties == other.min_properties {
            self.min_properties = None;
            other.min_properties = None;
        }

        // remove exact matches
        self.properties.retain(|self_key, self_val| {
            if let Some(other_val) = other.properties.get(self_key) {
                if self_val == other_val {
                    other.properties.shift_remove(self_key);
                    return false;
                }
            }

            true
        });

        // do partial exclusion
        for (self_key, self_val) in self.properties.iter_mut() {
            if let Some(other_val) = other.properties.shift_remove(self_key) {
                self_val.schema.not = Some(Box::new(other_val.schema));
                self_val.schema.exclude();
            }
        }

        let common_req = self
            .required
            .to_owned()
            .intersection(other.required.to_owned());

        self.required.retain(|r| !common_req.contains(r));
        other.required.retain(|r| !common_req.contains(r));

        for (k, prop) in self.properties.iter_mut() {
            prop.required = self.required.contains(k);
        }
    }

    pub fn merge(mut self, other: Self) -> Self {
        match (&self.max_properties, &other.max_properties) {
            (Some(self_max), Some(other_max)) if self_max > other_max => {
                self.max_properties = other.max_properties;
            }
            (None, Some(_)) => {
                self.max_properties = other.max_properties;
            }
            _ => {}
        }

        match (&self.min_properties, &other.min_properties) {
            (Some(self_min), Some(other_min)) if self_min < other_min => {
                self.min_properties = other.min_properties;
            }
            (None, Some(_)) => {
                self.min_properties = other.min_properties;
            }
            _ => {}
        }

        self.required = self.required.union(other.required);

        self.properties.extend(other.properties);

        for (k, prop) in self.properties.iter_mut() {
            prop.required = self.required.contains(k);
        }

        self.additional_properties = match (self.additional_properties, other.additional_properties)
        {
            (Some(self_add), Some(other_add)) => Some(Box::new(self_add.merge(*other_add))),
            (None, Some(other_add)) => Some(other_add),
            (Some(self_add), None) => Some(self_add),
            (None, None) => None,
        };

        self
    }

    pub fn from_value(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> Result<Self> {
        let mut properties = Map::new();

        let required = value
            .take("required")
            .and_then(Value::take_array)
            .map(|arr| {
                arr.into_iter()
                    .filter_map(Value::take_string)
                    .collect::<Set<_>>()
            })
            .unwrap_or_default();

        if let Some(props) = value.take("properties").and_then(Value::take_object) {
            for (k, v) in props.into_iter() {
                let is_prop_required = required.contains(&k);
                let mut prop = Property::try_parse(v, ctx, visited_refs, is_prop_required)?;
                prop.schema.metadata.field_name = Some(k.clone());

                properties.insert(k, Box::new(prop));
            }
        }

        let additional_properties = value
            .take("additionalProperties")
            .map(|v| Schema::try_parse(v, ctx, visited_refs, None))
            .transpose()?
            .map(Box::new);

        Ok(Self {
            max_properties: value.take("maxProperties").and_then(Value::take_number),
            min_properties: value.take("minProperties").and_then(Value::take_number),
            properties,
            additional_properties,
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
    fn parses_as_object() {
        let value = json!({
            "type": "object",
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object"
            }"#}
        );
    }

    #[test]
    fn parses_properties() {
        let value = json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "number",
                },
                "name": {
                    "type": "string",
                },
            },
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
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
    fn parses_additional_properties_as_object() {
        let value = json!({
            "type": "object",
            "additionalProperties": {
                "type": "string",
            },
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "additionalProperties": {
                "type": "string"
              }
            }"#}
        );
    }

    #[test]
    #[ignore]
    fn parses_additional_properties_as_boolean() {
        todo!()
    }

    #[test]
    fn parses_max_properties() {
        let value = json!({
            "type": "object",
            "maxProperties": 10,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "maxProperties": 10
            }"#}
        );
    }

    #[test]
    fn parses_min_properties() {
        let value = json!({
            "type": "object",
            "minProperties": 1,
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "minProperties": 1
            }"#}
        );
    }

    #[test]
    fn parses_required_when_properties() {
        let value = json!({
            "type": "object",
            "properties": {
                "id": {
                    "type": "number",
                },
                "name": {
                    "type": "string",
                },
            },
            "required": ["id", "name"],
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "required": [
                "id",
                "name"
              ],
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
    fn parses_default() {
        let value = json!({
            "type": "object",
            "default": {
                "id": 1,
                "name": "John",
            },
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
            {
              "type": "object",
              "default": {
                "id": 1,
                "name": "John"
              }
            }"#}
        );
    }

    #[test]
    fn parses_example() {
        let value = json!({
            "type": "object",
            "example": {
                "id": 1,
                "name": "John",
            },
        });

        let schema =
            Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

        assert_str_eq!(
            schema.pretty_print(),
            indoc! {r#"
          {
            "type": "object",
            "example": {
              "id": 1,
              "name": "John"
            }
          }"#}
        );
    }

    mod all_of {
        use super::*;

        #[test]
        fn merges_objects() {
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
                }"# }
            );
        }

        #[test]
        fn merges_additional_properties() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "additionalProperties": {
                    "type": "string"
                  }
                },
                {
                  "type": "object",
                  "additionalProperties": {
                    "type": "number"
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
                  "additionalProperties": {
                    "type": "string"
                  }
                }"# }
            );
        }

        #[test]
        fn merges_max_properties() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "maxProperties": 20
                },
                {
                  "type": "object",
                  "maxProperties": 10
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
                  "maxProperties": 10
                }"# }
            );
        }

        #[test]
        fn merges_max_properties_upper_end() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "maxProperties": 10
                },
                {
                  "type": "object",
                  "maxProperties": 20
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
                  "maxProperties": 10
                }"# }
            );
        }

        #[test]
        fn merges_min_properties() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "minProperties": 1
                },
                {
                  "type": "object",
                  "minProperties": 2
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
                  "minProperties": 2
                }"# }
            );
        }

        #[test]
        fn merges_min_properties_lower_end() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "minProperties": 2
                },
                {
                  "type": "object",
                  "minProperties": 1
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
                  "minProperties": 2
                }"# }
            );
        }

        #[test]
        fn merges_required() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "string"
                    },
                    "name": {
                      "type": "string"
                    }
                  },
                  "required": ["id"]
                },
                {
                  "type": "object",
                  "required": ["name"]
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
                  "required": [
                    "id",
                    "name"
                  ],
                  "properties": {
                    "id": {
                      "type": "string",
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
                }"# }
            );
        }

        #[test]
        fn merges_default() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "default": {
                    "id": 1
                  }
                },
                {
                  "type": "object",
                  "default": {
                    "name": "John"
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
                  "default": {
                    "name": "John"
                  }
                }"# }
            );
        }

        #[test]
        fn merges_properties() {
            let value = json!({
              "type": "object",
              "allOf": [
                {
                  "type": "object",
                  "properties": {
                    "id": {
                      "type": "number"
                    }
                  }
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

            let schema =
                Schema::try_parse(value, &ParserContext::default(), &mut Set::new(), None).unwrap();

            assert_str_eq!(
                schema.pretty_print(),
                indoc! { r#"
                {
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
                }"# }
            );
        }

        mod not {
            use super::*;

            #[test]
            fn merges_nots() {
                let value = json!({
                  "type": "object",
                  "allOf": [
                    {
                      "type": "object",
                      "not": {
                        "type": "object",
                        "maxProperties": 10
                      }
                    },
                    {
                      "type": "object",
                      "not": {
                        "type": "object",
                        "minProperties": 5
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
                      "not": {
                        "type": "object",
                        "maxProperties": 10,
                        "minProperties": 5
                      },
                      "type": "object"
                    }"# }
                );
            }

            #[test]
            fn excludes_max_properties_if_same() {
                let value = json!({
                  "type": "object",
                  "allOf": [
                    {
                      "type": "object",
                      "maxProperties": 10
                    },
                    {
                      "not": {
                        "type": "object",
                        "maxProperties": 10
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
                      "type": "object"
                    }"# }
                );
            }

            #[test]
            fn excludes_min_properties_if_same() {
                let value = json!({
                  "type": "object",
                  "allOf": [
                    {
                      "type": "object",
                      "minProperties": 10
                    },
                    {
                      "not": {
                        "type": "object",
                        "minProperties": 10
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
                      "type": "object"
                    }"# }
                );
            }

            #[test]
            fn excludes_required() {
                let value = json!({
                  "type": "object",
                  "allOf": [
                    {
                      "type": "object",
                      "properties": {
                        "id": {
                          "type": "string"
                        },
                        "name": {
                          "type": "string"
                        },
                      },
                      "required": ["id", "name"]
                    },
                    {
                      "not": {
                        "type": "object",
                        "required": ["id"]
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
                      "type": "object",
                      "required": [
                        "name"
                      ],
                      "properties": {
                        "id": {
                          "type": "string",
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
                    }"# }
                );
            }

            #[test]
            fn excludes_default() {
                let value = json!({
                  "type": "object",
                  "allOf": [
                    {
                      "type": "object",
                      "default": {
                        "id": 1,
                        "name": "John"
                      }
                    },
                    {
                      "not": {
                        "type": "object",
                        "default": {
                          "id": 1,
                          "name": "John"
                        }
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
                      "type": "object"
                    }"# }
                );
            }

            #[test]
            fn excludes_properties_exact() {
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
                        }
                      }
                    },
                    {
                      "not": {
                        "type": "object",
                        "properties": {
                          "id": {
                            "type": "number"
                          }
                        }
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
                      "type": "object",
                      "properties": {
                        "name": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "name"
                          }
                        }
                      }
                    }"# }
                );
            }

            #[test]
            fn excludes_properties_partial() {
                let value = json!({
                  "type": "object",
                  "allOf": [
                    {
                      "type": "object",
                      "properties": {
                        "user": {
                          "type": "object",
                          "properties": {
                            "parent": {
                              "type": "string"
                            },
                            "age": {
                              "type": "number"
                            }
                          }
                        },
                        "name": {
                          "type": "string"
                        }
                      }
                    },
                    {
                      "not": {
                        "type": "object",
                        "properties": {
                          "user": {
                            "type": "object",
                            "properties": {
                              "age": {
                                "type": "number"
                              }
                            }
                          }
                        }
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
                      "type": "object",
                      "properties": {
                        "user": {
                          "type": "object",
                          "properties": {
                            "parent": {
                              "type": "string",
                              "doctave_metadata": {
                                "field_name": "parent"
                              }
                            }
                          },
                          "doctave_metadata": {
                            "field_name": "user"
                          }
                        },
                        "name": {
                          "type": "string",
                          "doctave_metadata": {
                            "field_name": "name"
                          }
                        }
                      }
                    }"# }
                );
            }
        }
    }
}
