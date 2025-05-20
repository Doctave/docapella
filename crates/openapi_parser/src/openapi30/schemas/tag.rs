use super::external_documentation::ExternalDocumentation;
use crate::openapi30::parser;
use crate::{String, Value};

#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Tag object should contain a name"#)]
    MissingName,
    #[error(r#"Failed to parse external documentation: {0}"#)]
    ExternalDocumentationParseError(String),
}

#[derive(Debug, Clone)]
pub struct Tag {
    pub name: String,
    pub description: Option<String>,
    pub external_docs: Option<ExternalDocumentation>,
}

impl Tag {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let name = value
            .take("name")
            .and_then(Value::take_string)
            .ok_or(Error::MissingName)?;

        let description = value.take("description").and_then(Value::take_string);

        let external_docs = value
            .take("externalDocs")
            .map(ExternalDocumentation::try_parse)
            .transpose()?;

        Ok(Tag {
            name,
            description,
            external_docs,
        })
    }
}

impl From<String> for Tag {
    fn from(value: String) -> Self {
        Self {
            name: value,
            description: None,
            external_docs: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_name() {
        let value = json!({
            "name": "pet"
        });
        let tag = Tag::try_parse(value).unwrap();
        assert_str_eq!(
            tag.pretty_print(),
            indoc! {r#"
            {
              "name": "pet"
            }"#}
        );
    }

    #[test]
    fn parses_description() {
        let value = json!({
            "name": "pet",
            "description": "Everything about your Pets"
        });
        let tag = Tag::try_parse(value).unwrap();
        assert_str_eq!(
            tag.pretty_print(),
            indoc! {r#"
            {
              "name": "pet",
              "description": "Everything about your Pets"
            }"#}
        );
    }

    #[test]
    fn parses_external_docs() {
        let value = json!({
            "name": "pet",
            "externalDocs": {
                "description": "Find out more",
                "url": "http://swagger.io"
            }
        });
        let tag = Tag::try_parse(value).unwrap();
        assert_str_eq!(
            tag.pretty_print(),
            indoc! {r#"
            {
              "name": "pet",
              "externalDocs": {
                "url": "http://swagger.io",
                "description": "Find out more"
              }
            }"#}
        );
    }

    #[test]
    fn parses_all_fields() {
        let value = json!({
            "name": "pet",
            "description": "Everything about your Pets",
            "externalDocs": {
                "description": "Find out more",
                "url": "http://swagger.io"
            }
        });
        let tag = Tag::try_parse(value).unwrap();
        assert_str_eq!(
            tag.pretty_print(),
            indoc! {r#"
            {
              "name": "pet",
              "description": "Everything about your Pets",
              "externalDocs": {
                "url": "http://swagger.io",
                "description": "Find out more"
              }
            }"#}
        );
    }

    #[test]
    fn fails_on_missing_name() {
        let value = json!({
            "description": "Everything about your Pets"
        });
        let err = Tag::try_parse(value).unwrap_err();
        assert_str_eq!(err.to_string(), r#"Tag object should contain a name"#);
    }
}
