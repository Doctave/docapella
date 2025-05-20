use super::oauth_flow::OAuthFlow;
use crate::openapi30::parser;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Failed to parse OAuth flow: {0}"#)]
    OAuthFlowParseError(String),
}

#[derive(Debug, Clone)]
pub struct OAuthFlows {
    pub implicit: Option<OAuthFlow>,
    pub password: Option<OAuthFlow>,
    pub client_credentials: Option<OAuthFlow>,
    pub authorization_code: Option<OAuthFlow>,
}

impl OAuthFlows {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();
        to_string_pretty(&val).unwrap()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let implicit = value
            .take("implicit")
            .map(|v| OAuthFlow::try_parse(v, "implicit"))
            .transpose()
            .map_err(|e| Error::OAuthFlowParseError(e.to_string()))?;

        let password = value
            .take("password")
            .map(|v| OAuthFlow::try_parse(v, "password"))
            .transpose()
            .map_err(|e| Error::OAuthFlowParseError(e.to_string()))?;

        let client_credentials = value
            .take("clientCredentials")
            .map(|v| OAuthFlow::try_parse(v, "clientCredentials"))
            .transpose()
            .map_err(|e| Error::OAuthFlowParseError(e.to_string()))?;

        let authorization_code = value
            .take("authorizationCode")
            .map(|v| OAuthFlow::try_parse(v, "authorizationCode"))
            .transpose()
            .map_err(|e| Error::OAuthFlowParseError(e.to_string()))?;

        Ok(OAuthFlows {
            implicit,
            password,
            client_credentials,
            authorization_code,
        })
    }
}

#[cfg(test)]
mod test {
    use crate::{json, openapi30::schemas::oauth_flows::OAuthFlows};
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_oauth_flows() {
        let value = json!({
            "implicit": {
                "authorizationUrl": "https://example.com/oauth/authorize",
                "scopes": {
                    "write:pets": "modify pets in your account",
                    "read:pets": "read your pets"
                }
            },
            "authorizationCode": {
                "authorizationUrl": "https://example.com/oauth/authorize",
                "tokenUrl": "https://example.com/oauth/token",
                "scopes": {
                    "write:pets": "modify pets in your account",
                    "read:pets": "read your pets"
                }
            }
        });
        let oauth_flows = OAuthFlows::try_parse(value).unwrap();
        assert_str_eq!(
            oauth_flows.pretty_print(),
            indoc! {r#"
            {
              "implicit": {
                "authorizationUrl": "https://example.com/oauth/authorize",
                "scopes": {
                  "read:pets": "read your pets",
                  "write:pets": "modify pets in your account"
                }
              },
              "authorizationCode": {
                "authorizationUrl": "https://example.com/oauth/authorize",
                "tokenUrl": "https://example.com/oauth/token",
                "scopes": {
                  "read:pets": "read your pets",
                  "write:pets": "modify pets in your account"
                }
              }
            }"#}
        )
    }

    #[test]
    fn parses_oauth_flows_with_single_flow() {
        let value = json!({
            "clientCredentials": {
                "tokenUrl": "https://example.com/oauth/token",
                "scopes": {
                    "read:pets": "read your pets",
                    "write:pets": "modify pets in your account"
                }
            }
        });
        let oauth_flows = OAuthFlows::try_parse(value).unwrap();
        assert_str_eq!(
            oauth_flows.pretty_print(),
            indoc! {r#"
            {
              "clientCredentials": {
                "tokenUrl": "https://example.com/oauth/token",
                "scopes": {
                  "read:pets": "read your pets",
                  "write:pets": "modify pets in your account"
                }
              }
            }"#}
        )
    }

    #[test]
    fn fails_on_invalid_flow() {
        let value = json!({
            "implicit": {
                "scopes": {
                    "write:pets": "modify pets in your account",
                    "read:pets": "read your pets"
                }
            }
        });
        let err = OAuthFlows::try_parse(value).unwrap_err();
        assert!(err
            .to_string()
            .contains("Authorization URL is required for implicit and authorizationCode flows"));
    }
}
