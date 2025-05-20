use crate::openapi30::parser::ParserContext;
use crate::Operation;
use crate::Server;
use crate::Set;
use crate::String;

use crate::openapi30::parser;
use crate::Value;

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Clone)]
pub struct Webhook {
    pub operation: Operation,
    pub name: String,
}

impl Webhook {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
        path_or_root_servers: &[Server],
        name: String,
    ) -> parser::Result<Self> {
        let operation = Operation::try_parse(
            value,
            ctx,
            visited_refs,
            "webhook".into(),
            "".into(),
            vec![],
            path_or_root_servers,
        )?;

        Ok(Self { operation, name })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use crate::json;

    #[test]
    fn parses_webhook() {
        let value = json!({
          "operationId": "instrument_event",
          "summary": "Instruments events",
          "description": "Instruments events",
          "tags": ["Instruments"],
          "requestBody": {
              "content": {
                  "application/json": {
                      "schema": {
                          "title": "Webhook - Instruments - Callback",
                          "type": "object",
                          "properties": {
                              "id": {
                                  "type": "string",
                                  "format": "uuid",
                                  "description": "Event unique identifier"
                              }
                          }
                      }
                  }
              }
          }
        });

        let webhook = Webhook::try_parse(
            value,
            &ParserContext::default(),
            &mut Set::new(),
            &[],
            "Some webhook".into(),
        )
        .unwrap();

        assert_str_eq!(
            webhook.pretty_print(),
            indoc! {r#"
            {
              "Some webhook": {
                "summary": "Instruments events",
                "description": "Instruments events",
                "tags": [
                  "Instruments"
                ],
                "operationId": "instrument_event",
                "requestBody": {
                  "content": {
                    "application/json": {
                      "schema": {
                        "title": "Webhook - Instruments - Callback",
                        "type": "object",
                        "properties": {
                          "id": {
                            "description": "Event unique identifier",
                            "type": "string",
                            "format": "uuid",
                            "doctave_metadata": {
                              "field_name": "id"
                            }
                          }
                        },
                        "doctave_metadata": {
                          "title": "Webhook - Instruments - Callback"
                        }
                      }
                    }
                  }
                },
                "method": "webhook"
              }
            }"# }
        )
    }
}
