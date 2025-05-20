use super::oauth_flows::OAuthFlows;
use crate::openapi30::parser;
use crate::openapi30::parser::ParserContext;
use crate::Set;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"SecurityScheme object should contain a type"#)]
    MissingType,
    #[error(r#"Invalid security scheme type: {0}"#)]
    InvalidType(String),
    #[error(r#"Failed to parse OAuth flows: {0}"#)]
    OAuthFlowsParseError(String),
}

#[derive(Debug, Clone)]
pub struct SecurityScheme {
    pub kind: SecuritySchemeKind,
    pub description: Option<String>,
}

#[derive(Debug, Clone)]
pub enum SecuritySchemeKind {
    ApiKey(ApiKeySecurityScheme),
    Http(HttpSecurityScheme),
    OAuth2(Box<OAuth2SecurityScheme>),
    OpenIdConnect(OpenIdConnectSecurityScheme),
}

#[derive(Debug, Clone)]
pub struct ApiKeySecurityScheme {
    pub name: String,
    pub r#in: String,
}

#[derive(Debug, Clone)]
pub struct HttpSecurityScheme {
    pub scheme: String,
    pub bearer_format: Option<String>,
}

#[derive(Debug, Clone)]
pub struct OAuth2SecurityScheme {
    pub flows: OAuthFlows,
}

#[derive(Debug, Clone)]
pub struct OpenIdConnectSecurityScheme {
    pub open_id_connect_url: String,
}

impl SecurityScheme {
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
        let mut value = value
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|ref_path| ctx.get(ref_path, visited_refs))
            .unwrap_or(value);

        let r#type = value
            .take("type")
            .and_then(Value::take_string)
            .ok_or(Error::MissingType)?;

        let description = value.take("description").and_then(Value::take_string);

        let kind = match r#type.as_str() {
            "apiKey" => {
                let name = value
                    .take("name")
                    .and_then(Value::take_string)
                    .ok_or(Error::InvalidType("apiKey missing 'name'".into()))?;
                let r#in = value
                    .take("in")
                    .and_then(Value::take_string)
                    .ok_or(Error::InvalidType("apiKey missing 'in'".into()))?;
                SecuritySchemeKind::ApiKey(ApiKeySecurityScheme { name, r#in })
            }
            "http" => {
                let scheme = value
                    .take("scheme")
                    .and_then(Value::take_string)
                    .ok_or(Error::InvalidType("http missing 'scheme'".into()))?;
                let bearer_format = value.take("bearerFormat").and_then(Value::take_string);
                SecuritySchemeKind::Http(HttpSecurityScheme {
                    scheme,
                    bearer_format,
                })
            }
            "oauth2" => {
                let flows = value
                    .take("flows")
                    .ok_or(Error::InvalidType("oauth2 missing 'flows'".into()))
                    .and_then(|v| {
                        OAuthFlows::try_parse(v)
                            .map_err(|e| Error::OAuthFlowsParseError(String::from(e.to_string())))
                    })?;
                SecuritySchemeKind::OAuth2(Box::new(OAuth2SecurityScheme { flows }))
            }
            "openIdConnect" => {
                let open_id_connect_url = value
                    .take("openIdConnectUrl")
                    .and_then(Value::take_string)
                    .ok_or(Error::InvalidType(
                        "openIdConnect missing 'openIdConnectUrl'".into(),
                    ))?;
                SecuritySchemeKind::OpenIdConnect(OpenIdConnectSecurityScheme {
                    open_id_connect_url,
                })
            }
            _ => return Err(Error::InvalidType(r#type).into()),
        };

        Ok(SecurityScheme { kind, description })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use parser::ParserContext;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_api_key_security_scheme() {
        let value = json!({
            "type": "apiKey",
            "name": "api_key",
            "in": "header"
        });
        let security_scheme =
            SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            security_scheme.pretty_print(),
            indoc! {r#"
            {
              "type": "apiKey",
              "in": "header",
              "name": "api_key"
            }"#}
        );
    }

    #[test]
    fn parses_http_security_scheme() {
        let value = json!({
            "type": "http",
            "scheme": "bearer",
        });
        let security_scheme =
            SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            security_scheme.pretty_print(),
            indoc! {r#"
            {
              "type": "http",
              "scheme": "bearer"
            }"#}
        );
    }

    #[test]
    fn parses_http_security_scheme_bearer_format() {
        let value = json!({
            "type": "http",
            "scheme": "bearer",
            "bearerFormat": "JWT"
        });
        let security_scheme =
            SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            security_scheme.pretty_print(),
            indoc! {r#"
            {
              "type": "http",
              "scheme": "bearer",
              "bearerFormat": "JWT"
            }"#}
        );
    }

    #[test]
    fn parses_oauth2_security_scheme() {
        let value = json!({
            "type": "oauth2",
            "flows": {
                "implicit": {
                    "authorizationUrl": "https://example.com/api/oauth/dialog",
                    "scopes": {
                        "read:pets": "read your pets",
                        "write:pets": "modify pets in your account"
                    }
                }
            }
        });
        let security_scheme =
            SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            security_scheme.pretty_print(),
            indoc! {r#"
            {
              "type": "oauth2",
              "flows": {
                "implicit": {
                  "authorizationUrl": "https://example.com/api/oauth/dialog",
                  "scopes": {
                    "read:pets": "read your pets",
                    "write:pets": "modify pets in your account"
                  }
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_open_id_connect_security_scheme() {
        let value = json!({
            "type": "openIdConnect",
            "openIdConnectUrl": "https://example.com/.well-known/openid-configuration"
        });
        let security_scheme =
            SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            security_scheme.pretty_print(),
            indoc! {r#"
            {
              "type": "openIdConnect",
              "openIdConnectUrl": "https://example.com/.well-known/openid-configuration"
            }"#}
        );
    }

    #[test]
    fn fails_on_missing_type() {
        let value = json!({
            "name": "api_key",
            "in": "header"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"SecurityScheme object should contain a type"#
        );
    }

    #[test]
    fn fails_on_invalid_type() {
        let value = json!({
            "type": "invalid_type"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Invalid security scheme type: invalid_type"#
        );
    }

    #[test]
    fn fails_on_missing_name_for_api_key() {
        let value = json!({
            "type": "apiKey",
            "in": "header"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Invalid security scheme type: apiKey missing 'name'"#
        );
    }

    #[test]
    fn fails_on_missing_in_for_api_key() {
        let value = json!({
            "type": "apiKey",
            "name": "api_key"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Invalid security scheme type: apiKey missing 'in'"#
        );
    }

    #[test]
    fn fails_on_missing_scheme_for_http() {
        let value = json!({
            "type": "http"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Invalid security scheme type: http missing 'scheme'"#
        );
    }

    #[test]
    fn fails_on_missing_flows_for_oauth2() {
        let value = json!({
            "type": "oauth2"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Invalid security scheme type: oauth2 missing 'flows'"#
        );
    }

    #[test]
    fn fails_on_missing_open_id_connect_url() {
        let value = json!({
            "type": "openIdConnect"
        });
        let err = SecurityScheme::try_parse(value, &ParserContext::default(), &mut Set::new())
            .unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Invalid security scheme type: openIdConnect missing 'openIdConnectUrl'"#
        );
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/securitySchemes/api_key"
        });

        let ref_cache = json!({
          "securitySchemes": {
            "api_key": {
                "type": "apiKey",
                "name": "api_key",
                "in": "header"
            }
          }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(ref_cache);

        let security_scheme = SecurityScheme::try_parse(value, &ctx, &mut Set::new()).unwrap();
        assert_str_eq!(
            security_scheme.pretty_print(),
            indoc! {r#"
            {
              "type": "apiKey",
              "in": "header",
              "name": "api_key"
            }"#}
        );
    }
}
