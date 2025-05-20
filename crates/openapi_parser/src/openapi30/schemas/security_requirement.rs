use crate::openapi30::parser;
use crate::Map;
use crate::String;
use crate::Value;

#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid security requirement format"#)]
    InvalidFormat,
}

#[derive(Debug, Clone)]
pub struct SecurityRequirement {
    pub requirements: Map<String, Vec<String>>,
}

impl SecurityRequirement {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(value: Value) -> parser::Result<Self> {
        let requirements = value
            .take_object()
            .ok_or(Error::InvalidFormat)?
            .into_iter()
            .map(|(key, value)| {
                let scopes = value
                    .take_array()
                    .ok_or(Error::InvalidFormat)?
                    .into_iter()
                    .filter_map(Value::take_string)
                    .collect();
                Ok((key, scopes))
            })
            .collect::<Result<Map<_, _>, parser::Error>>()?;

        Ok(SecurityRequirement { requirements })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_empty_security_requirement() {
        let value = json!({});
        let security_requirement = SecurityRequirement::try_parse(value).unwrap();
        assert_str_eq!(
            security_requirement.pretty_print(),
            indoc! {r#"
            {}"#}
        );
    }

    #[test]
    fn parses_security_requirement_with_empty_scopes() {
        let value = json!({
            "api_key": []
        });
        let security_requirement = SecurityRequirement::try_parse(value).unwrap();
        assert_str_eq!(
            security_requirement.pretty_print(),
            indoc! {r#"
            {
              "api_key": []
            }"#}
        );
    }

    #[test]
    fn parses_security_requirement_with_scopes() {
        let value = json!({
            "petstore_auth": [
                "write:pets",
                "read:pets"
            ]
        });
        let security_requirement = SecurityRequirement::try_parse(value).unwrap();
        assert_str_eq!(
            security_requirement.pretty_print(),
            indoc! {r#"
            {
              "petstore_auth": [
                "write:pets",
                "read:pets"
              ]
            }"#}
        );
    }

    #[test]
    fn parses_multiple_security_requirements() {
        let value = json!({
            "api_key": [],
            "petstore_auth": [
                "write:pets",
                "read:pets"
            ]
        });
        let security_requirement = SecurityRequirement::try_parse(value).unwrap();
        assert_str_eq!(
            security_requirement.pretty_print(),
            indoc! {r#"
            {
              "api_key": [],
              "petstore_auth": [
                "write:pets",
                "read:pets"
              ]
            }"#}
        );
    }

    #[test]
    fn fails_on_invalid_format() {
        let value = json!(["invalid_format"]);
        let err = SecurityRequirement::try_parse(value).unwrap_err();
        assert_str_eq!(err.to_string(), r#"Invalid security requirement format"#);
    }

    #[test]
    fn fails_on_invalid_scopes_format() {
        let value = json!({
            "api_key": "invalid_scopes_format"
        });
        let err = SecurityRequirement::try_parse(value).unwrap_err();
        assert_str_eq!(err.to_string(), r#"Invalid security requirement format"#);
    }
}
