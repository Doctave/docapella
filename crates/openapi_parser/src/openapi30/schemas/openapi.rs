use crate::{openapi30::parser::ParserContext, String};

use crate::Set;
use crate::Value;
use crate::{Map, Operation};
use thiserror::Error;

use crate::openapi30::parser;

use super::{
    components::Components, external_documentation::ExternalDocumentation, info::Info,
    path_item::PathItem, security_requirement::SecurityRequirement, server::Server, tag::Tag,
    webhook::Webhook,
};

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"openapi field should be a string"#)]
    InvalidOpenAPI,
    #[error(r#"Missing required field `openapi`"#)]
    MissingOpenAPI,
    #[error(r#"Missing required field `info`"#)]
    MissingInfo,
    #[error(r#"Missing required field `paths`"#)]
    MissingPaths,
}

#[derive(Debug, Clone)]
pub struct OpenAPI {
    pub openapi: String,
    pub info: Info,
    pub servers: Vec<Server>,
    pub paths: Map<String, PathItem>,
    pub components: Option<Components>,
    pub security: Vec<SecurityRequirement>,
    pub tags: Vec<Tag>,
    pub external_docs: Option<ExternalDocumentation>,
    pub webhooks: Vec<Webhook>,
}

impl OpenAPI {
    #[cfg(test)]
    pub(crate) fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn tag_names(&self) -> Vec<std::string::String> {
        let mut all_tags = vec![];

        all_tags.extend(
            self.tags
                .iter()
                .map(|t| t.name.to_string())
                .collect::<Vec<_>>(),
        );

        let inline_tags = self
            .operations()
            .iter()
            .flat_map(|op| op.tags.iter().map(|t| t.to_string()).collect::<Vec<_>>())
            .collect::<Vec<_>>();

        for inline_tag in &inline_tags {
            if !all_tags.iter().any(|t| t == inline_tag) {
                all_tags.push(inline_tag.to_string());
            }
        }

        all_tags
    }

    pub fn operations(&self) -> Vec<&Operation> {
        self.paths
            .iter()
            .flat_map(|(_, item)| item.operations())
            .collect()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let mut ctx = ParserContext::default();
        let mut visited_refs: Set<String> = Set::new();

        let openapi = value
            .take("openapi")
            .ok_or(Error::MissingOpenAPI)?
            .take_string()
            .ok_or(Error::InvalidOpenAPI)?;

        let mut components = None;

        if let Some(c) = value.take("components") {
            ctx.ref_cache.with_value(c.clone());

            components = Some(Components::try_parse(c, &ctx, &mut visited_refs)?);
        }

        let mut servers = vec![];
        if let Some(_servers) = value.take("servers").and_then(Value::take_array) {
            for server in _servers {
                servers.push(Server::try_parse(server)?)
            }
        }

        let info = Info::try_parse(value.take("info").ok_or(Error::MissingInfo)?, &ctx)?;

        let paths = value
            .take("paths")
            .and_then(Value::take_object)
            .unwrap_or_default()
            .into_iter()
            .map(|(k, v)| {
                Ok((
                    k.clone(),
                    PathItem::try_parse(v, &ctx, &mut visited_refs, k, &servers)?,
                ))
            })
            .collect::<parser::Result<Map<String, PathItem>>>()?;

        let security = value
            .take("security")
            .and_then(Value::take_array)
            .unwrap_or_default()
            .into_iter()
            .map(SecurityRequirement::try_parse)
            .collect::<parser::Result<Vec<SecurityRequirement>>>()?;

        let tags = value
            .take("tags")
            .and_then(Value::take_array)
            .unwrap_or_default()
            .into_iter()
            .map(Tag::try_parse)
            .collect::<parser::Result<Vec<Tag>>>()?;

        let external_docs = value
            .take("externalDocs")
            .map(ExternalDocumentation::try_parse)
            .transpose()?;

        let webhooks = value
            .take("x-webhooks")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .flat_map(|(k, v)| {
                        let obj = v.take_object().unwrap_or_default();

                        obj.into_values()
                            .map(|v| {
                                Webhook::try_parse(v, &ctx, &mut visited_refs, &servers, k.clone())
                            })
                            .collect::<Vec<_>>()
                    })
                    .collect::<parser::Result<Vec<_>>>()
            })
            .transpose()?
            .unwrap_or_default();

        Ok(Self {
            openapi,
            info,
            servers,
            paths,
            components,
            security,
            tags,
            external_docs,
            webhooks,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::json;

    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_openapi() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          }
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! {r#"
            {
              "openapi": "3.0.0",
              "info": {
                "title": "Example",
                "version": "1.0.0"
              }
            }"# }
        );
    }

    #[test]
    fn fails_on_missing_openapi() {
        let value = json!({
          "info": {
            "title": "Example",
            "version": "1.0.0"
          }
        });

        let err = super::OpenAPI::try_parse(value).unwrap_err();

        assert_eq!(err.to_string(), "Missing required field `openapi`");
    }

    #[test]
    fn fails_on_invalid_openapi() {
        let value = json!({
          "openapi": 3,
          "info": {
            "title": "Example",
            "version": "1.0.0"
          }
        });

        let err = super::OpenAPI::try_parse(value).unwrap_err();

        assert_eq!(err.to_string(), "openapi field should be a string");
    }

    #[test]
    fn fails_on_missing_info() {
        let value = json!({
          "openapi": "3.0.0"
        });

        let err = super::OpenAPI::try_parse(value).unwrap_err();

        assert_eq!(err.to_string(), "Missing required field `info`");
    }

    #[test]
    fn parses_openapi_with_servers() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "servers": [
            {
              "url": "https://api.example.com"
            }
          ]
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! { r#"
          {
            "openapi": "3.0.0",
            "info": {
              "title": "Example",
              "version": "1.0.0"
            },
            "servers": [
              {
                "url": "https://api.example.com"
              }
            ]
          }"# }
        )
    }

    #[test]
    fn parses_openapi_with_paths() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "paths": {
            "/": {
              "get": {
                "responses": {
                  "200": {
                    "description": "OK"
                  }
                }
              }
            }
          }
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! { r#"
            {
              "openapi": "3.0.0",
              "info": {
                "title": "Example",
                "version": "1.0.0"
              },
              "paths": {
                "/": {
                  "get": {
                    "responses": {
                      "200": {
                        "description": "OK"
                      }
                    },
                    "method": "get"
                  }
                }
              }
            }"# }
        );
    }

    #[test]
    #[ignore]
    fn parses_openapi_with_components() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "components": {
            "schemas": {
              "Example": {
                "type": "object"
              }
            }
          }
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert!(openapi.components.is_some());
    }

    #[test]
    fn parses_openapi_with_security() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "security": [
            {
              "api_key": []
            }
          ]
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! { r#"
            {
              "openapi": "3.0.0",
              "info": {
                "title": "Example",
                "version": "1.0.0"
              },
              "security": [
                {
                  "api_key": []
                }
              ]
            }"# }
        );
    }

    #[test]
    fn parses_openapi_with_tags() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "tags": [
            {
              "name": "Example"
            }
          ]
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! { r#"
            {
              "openapi": "3.0.0",
              "info": {
                "title": "Example",
                "version": "1.0.0"
              },
              "tags": [
                {
                  "name": "Example"
                }
              ]
            }"# }
        );
    }

    #[test]
    fn parses_openapi_with_external_docs() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "externalDocs": {
            "url": "https://example.com"
          }
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! { r#"
            {
              "openapi": "3.0.0",
              "info": {
                "title": "Example",
                "version": "1.0.0"
              },
              "externalDocs": {
                "url": "https://example.com"
              }
            }"# }
        );
    }

    #[test]
    fn parses_openapi_with_webhooks() {
        let value = json!({
          "openapi": "3.0.0",
          "info": {
            "title": "Example",
            "version": "1.0.0"
          },
          "x-webhooks": {
            "foo": {
              "post": {
                "operationId": "foo",
                "requestBody": {
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
                    "description": "OK"
                  }
                }
              }
            }
          }
        });

        let openapi = super::OpenAPI::try_parse(value).unwrap();

        assert_str_eq!(
            openapi.pretty_print(),
            indoc! { r#"
            {
              "openapi": "3.0.0",
              "info": {
                "title": "Example",
                "version": "1.0.0"
              },
              "x-webhooks": {
                "foo": {
                  "operationId": "foo",
                  "responses": {
                    "200": {
                      "description": "OK"
                    }
                  },
                  "requestBody": {
                    "content": {
                      "application/json": {
                        "schema": {
                          "type": "object"
                        }
                      }
                    }
                  },
                  "method": "webhook"
                }
              }
            }"# }
        );
    }
}
