use crate::openapi30::parser;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"ExternalDocumentation object should contain a url"#)]
    MissingUrl,
}

#[derive(Debug, Clone)]
pub struct ExternalDocumentation {
    pub description: Option<String>,
    pub url: String,
}

impl ExternalDocumentation {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let description = value.take("description").and_then(Value::take_string);
        let url = value
            .take("url")
            .and_then(Value::take_string)
            .ok_or(Error::MissingUrl)?;

        Ok(ExternalDocumentation { description, url })
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_url() {
        let value = json!({
            "url": "https://example.com/docs"
        });
        let external_docs = ExternalDocumentation::try_parse(value).unwrap();

        assert_str_eq!(
            external_docs.pretty_print(),
            indoc! {r#"
            {
              "url": "https://example.com/docs"
            }"#}
        );
    }

    #[test]
    fn parses_description() {
        let value = json!({
            "url": "https://example.com/docs",
            "description": "Find more info here"
        });
        let external_docs = ExternalDocumentation::try_parse(value).unwrap();

        assert_str_eq!(
            external_docs.pretty_print(),
            indoc! {r#"
            {
              "url": "https://example.com/docs",
              "description": "Find more info here"
            }"#}
        );
    }

    #[test]
    fn fails_on_missing_url() {
        let value = json!({
            "description": "Find more info here"
        });
        let err = ExternalDocumentation::try_parse(value).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"ExternalDocumentation object should contain a url"#
        );
    }
}
