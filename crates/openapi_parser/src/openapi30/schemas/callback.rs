use super::path_item::PathItem;
use crate::openapi30::parser::{self, ParserContext};
use crate::Map;
use crate::{String, Value};

use crate::Set;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Callback object should contain at least one callback"#)]
    EmptyCallback,
    #[error(r#"Failed to parse PathItem: {0}"#)]
    PathItemParseError(String),
}

#[derive(Debug, Clone)]
pub struct Callback {
    pub callbacks: Map<String, PathItem>,
}

impl Callback {
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
        let value = value
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|ref_path| ctx.get(ref_path, &mut visited_refs))
            .unwrap_or(value);

        let callbacks = value
            .take_object()
            .ok_or(Error::EmptyCallback)?
            .into_iter()
            .map(|(k, v)| {
                PathItem::try_parse(v, ctx, &mut visited_refs, k.clone(), &[])
                    .map(|path_item| (k, path_item))
            })
            .collect::<Result<Map<_, _>, _>>()?;

        if callbacks.is_empty() {
            return Err(Error::EmptyCallback.into());
        }

        Ok(Callback { callbacks })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_single_path() {
        let value = json!({
            "{$request.query.queryUrl}": {
                "post": {
                    "requestBody": {
                        "description": "Callback payload",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "callback successfully processed"
                        }
                    }
                }
            }
        });
        let callback =
            Callback::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            callback.pretty_print(),
            indoc! {r#"
            {
              "{$request.query.queryUrl}": {
                "post": {
                  "responses": {
                    "200": {
                      "description": "callback successfully processed"
                    }
                  },
                  "requestBody": {
                    "description": "Callback payload",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object"
                        }
                      }
                    }
                  },
                  "method": "post"
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_multiple_paths() {
        let value = json!({
            "{$request.query.queryUrl}": {
                "post": {
                    "requestBody": {
                        "description": "Callback payload",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "callback successfully processed"
                        }
                    }
                }
            },
            "{$request.query.alternateUrl}": {
                "put": {
                    "requestBody": {
                        "description": "Alternate callback payload",
                        "content": {
                            "application/json": {
                                "schema": {
                                    "type": "object"
                                }
                            }
                        }
                    },
                    "responses": {
                        "200": {
                            "description": "alternate callback successfully processed"
                        }
                    }
                }
            }
        });
        let callback =
            Callback::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            callback.pretty_print(),
            indoc! {r#"
            {
              "{$request.query.queryUrl}": {
                "post": {
                  "responses": {
                    "200": {
                      "description": "callback successfully processed"
                    }
                  },
                  "requestBody": {
                    "description": "Callback payload",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object"
                        }
                      }
                    }
                  },
                  "method": "post"
                }
              },
              "{$request.query.alternateUrl}": {
                "put": {
                  "responses": {
                    "200": {
                      "description": "alternate callback successfully processed"
                    }
                  },
                  "requestBody": {
                    "description": "Alternate callback payload",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object"
                        }
                      }
                    }
                  },
                  "method": "put"
                }
              }
            }"#}
        );
    }

    #[test]
    fn fails_on_empty_callback() {
        let value = json!({});
        let err =
            Callback::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Callback object should contain at least one callback"#
        );
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/callbacks/Callback"
        });

        let ref_cache = json!({
          "callbacks": {
            "Callback": {
              "{$request.query.queryUrl}": {
                  "post": {
                      "requestBody": {
                          "description": "Callback payload",
                          "content": {
                              "application/json": {
                                  "schema": {
                                      "type": "object"
                                  }
                              }
                          }
                      },
                      "responses": {
                          "200": {
                              "description": "callback successfully processed"
                          }
                      }
                  }
              }
            }
          }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(ref_cache);

        let callback = Callback::try_parse(value, &ctx, &mut Set::new()).unwrap();
        assert_str_eq!(
            callback.pretty_print(),
            indoc! {r#"
            {
              "{$request.query.queryUrl}": {
                "post": {
                  "responses": {
                    "200": {
                      "description": "callback successfully processed"
                    }
                  },
                  "requestBody": {
                    "description": "Callback payload",
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object"
                        }
                      }
                    }
                  },
                  "method": "post"
                }
              },
              "doctave_metadata": {}
            }"#}
        );
    }
}
