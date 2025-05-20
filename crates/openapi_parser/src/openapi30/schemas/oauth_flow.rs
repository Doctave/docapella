use crate::openapi30::parser;
use crate::Map;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"OAuthFlow object should contain scopes"#)]
    MissingScopes,
    #[error(r#"Authorization URL is required for implicit and authorizationCode flows"#)]
    MissingAuthorizationUrl,
    #[error(
        r#"Token URL is required for password, clientCredentials, and authorizationCode flows"#
    )]
    MissingTokenUrl,
    #[error(r#"Failed to parse OAuth flow"#)]
    OAuthFlowParseError,
}

#[derive(Debug, Clone)]
pub enum OAuthFlow {
    Implicit {
        authorization_url: String,
        refresh_url: Option<String>,
        scopes: Map<String, String>,
    },
    Password {
        token_url: String,
        refresh_url: Option<String>,
        scopes: Map<String, String>,
    },
    ClientCredentials {
        token_url: String,
        refresh_url: Option<String>,
        scopes: Map<String, String>,
    },
    AuthorizationCode {
        authorization_url: String,
        refresh_url: Option<String>,
        token_url: String,
        scopes: Map<String, String>,
    },
}

impl OAuthFlow {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value, flow_type: &str) -> parser::Result<Self> {
        let authorization_url = value.take("authorizationUrl").and_then(Value::take_string);
        let token_url = value.take("tokenUrl").and_then(Value::take_string);
        let refresh_url = value.take("refreshUrl").and_then(Value::take_string);

        let scopes = value
            .take("scopes")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .filter_map(|(k, v)| v.take_string().map(|v| (k, v)))
                    .collect()
            })
            .ok_or(Error::MissingScopes)?;

        match flow_type {
            "implicit" => {
                let authorization_url = authorization_url.ok_or(Error::MissingAuthorizationUrl)?;

                Ok(OAuthFlow::Implicit {
                    authorization_url,
                    refresh_url,
                    scopes,
                })
            }
            "password" => {
                let token_url = token_url.ok_or(Error::MissingTokenUrl)?;

                Ok(OAuthFlow::Password {
                    token_url,
                    refresh_url,
                    scopes,
                })
            }
            "clientCredentials" => {
                let token_url = token_url.ok_or(Error::MissingTokenUrl)?;

                Ok(OAuthFlow::ClientCredentials {
                    token_url,
                    refresh_url,
                    scopes,
                })
            }
            "authorizationCode" => {
                let authorization_url = authorization_url.ok_or(Error::MissingAuthorizationUrl)?;
                let token_url = token_url.ok_or(Error::MissingTokenUrl)?;

                Ok(OAuthFlow::AuthorizationCode {
                    authorization_url,
                    token_url,
                    refresh_url,
                    scopes,
                })
            }
            _ => Err(Error::OAuthFlowParseError.into()),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{json, openapi30::schemas::oauth_flow::OAuthFlow};
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_implicit_flow() {
        let value = json!({
            "authorizationUrl": "https://example.com/oauth/authorize",
            "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
            }
        });
        let oauth_flow = OAuthFlow::try_parse(value, "implicit").unwrap();
        assert_str_eq!(
            oauth_flow.pretty_print(),
            indoc! {r#"
            {
              "authorizationUrl": "https://example.com/oauth/authorize",
              "scopes": {
                "read:pets": "read your pets",
                "write:pets": "modify pets in your account"
              }
            }"#}
        )
    }

    #[test]
    fn parses_authorization_code_flow() {
        let value = json!({
            "authorizationUrl": "https://example.com/oauth/authorize",
            "tokenUrl": "https://example.com/oauth/token",
            "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
            }
        });
        let oauth_flow = OAuthFlow::try_parse(value, "authorizationCode").unwrap();
        assert_str_eq!(
            oauth_flow.pretty_print(),
            indoc! {r#"
            {
              "authorizationUrl": "https://example.com/oauth/authorize",
              "tokenUrl": "https://example.com/oauth/token",
              "scopes": {
                "read:pets": "read your pets",
                "write:pets": "modify pets in your account"
              }
            }"#}
        )
    }

    #[test]
    fn parses_client_credentials_flow() {
        let value = json!({
            "tokenUrl": "https://example.com/oauth/token",
            "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
            }
        });
        let oauth_flow = OAuthFlow::try_parse(value, "clientCredentials").unwrap();
        assert_str_eq!(
            oauth_flow.pretty_print(),
            indoc! {r#"
            {
              "tokenUrl": "https://example.com/oauth/token",
              "scopes": {
                "read:pets": "read your pets",
                "write:pets": "modify pets in your account"
              }
            }"#}
        )
    }

    #[test]
    fn missing_scopes() {
        let value = json!({
            "authorizationUrl": "https://example.com/oauth/authorize",
            "tokenUrl": "https://example.com/oauth/token"
        });
        let err = OAuthFlow::try_parse(value, "authorizationCode").unwrap_err();
        assert_str_eq!(err.to_string(), r#"OAuthFlow object should contain scopes"#)
    }

    #[test]
    fn missing_authorization_url_for_implicit() {
        let value = json!({
            "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
            }
        });
        let err = OAuthFlow::try_parse(value, "implicit").unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Authorization URL is required for implicit and authorizationCode flows"#
        )
    }

    #[test]
    fn missing_token_url_for_client_credentials() {
        let value = json!({
            "scopes": {
                "write:pets": "modify pets in your account",
                "read:pets": "read your pets"
            }
        });
        let err = OAuthFlow::try_parse(value, "clientCredentials").unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Token URL is required for password, clientCredentials, and authorizationCode flows"#
        )
    }
}
