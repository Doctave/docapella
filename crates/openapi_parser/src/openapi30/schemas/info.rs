use crate::openapi30::parser::ParserContext;
use crate::String;

use crate::Value;
use thiserror::Error;

use crate::openapi30::parser;

use super::{contact::Contact, license::License};

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Info object should contain a title"#)]
    MissingTitle,
    #[error(r#"Info object should contain a version"#)]
    MissingVersion,
}

#[derive(Debug, Clone)]
pub struct Info {
    pub title: String,
    pub description: Option<String>,
    pub terms_of_service: Option<String>,
    pub contact: Option<Contact>,
    pub license: Option<License>,
    pub version: String,
}

impl Info {
    #[cfg(test)]
    pub(crate) fn pretty_print(self) -> String {
        let val: Value = self.into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value, _ctx: &ParserContext) -> parser::Result<Self> {
        let title = value
            .take("title")
            .and_then(|s| s.take_string())
            .ok_or(Error::MissingTitle)?;

        let version = value
            .take("version")
            .and_then(|s| s.take_string())
            .ok_or(Error::MissingVersion)?;

        let description = value.take("description").and_then(|s| s.take_string());

        let terms_of_service = value.take("termsOfService").and_then(|s| s.take_string());
        let contact = value.take("contact").map(Contact::try_parse).transpose()?;
        let license = value.take("license").map(License::try_parse).transpose()?;

        Ok(Self {
            title,
            description,
            terms_of_service,
            contact,
            license,
            version,
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
    fn parses() {
        let value = json!({
          "title": "foo",
          "version": "1.0.0"
        });

        let info = Info::try_parse(value, &ParserContext::default()).unwrap();

        assert_str_eq!(
            info.pretty_print(),
            indoc! {r#"
            {
              "title": "foo",
              "version": "1.0.0"
            }"# }
        );
    }

    #[test]
    fn parses_with_description() {
        let value = json!({
          "title": "foo",
          "version": "1.0.0",
          "description": "bar"
        });

        let info = Info::try_parse(value, &ParserContext::default()).unwrap();

        assert_str_eq!(
            info.pretty_print(),
            indoc! {r#"
            {
              "title": "foo",
              "description": "bar",
              "version": "1.0.0"
            }"# }
        );
    }

    #[test]
    fn parses_with_terms_of_service() {
        let value = json!({
          "title": "foo",
          "version": "1.0.0",
          "termsOfService": "http://example.com"
        });

        let info = Info::try_parse(value, &ParserContext::default()).unwrap();

        assert_str_eq!(
            info.pretty_print(),
            indoc! {r#"
            {
              "title": "foo",
              "termsOfService": "http://example.com",
              "version": "1.0.0"
            }"# }
        );
    }

    #[test]
    fn parses_with_contact() {
        let value = json!({
          "title": "foo",
          "version": "1.0.0",
          "contact": {
            "name": "bar",
            "email": "baz"
          }
        });

        let info = Info::try_parse(value, &ParserContext::default()).unwrap();

        assert_str_eq!(
            info.pretty_print(),
            indoc! {r#"
            {
              "title": "foo",
              "contact": {
                "name": "bar",
                "email": "baz"
              },
              "version": "1.0.0"
            }"# }
        );
    }

    #[test]
    fn parses_with_license() {
        let value = json!({
          "title": "foo",
          "version": "1.0.0",
          "license": {
            "name": "bar",
            "url": "baz"
          }
        });

        let info = Info::try_parse(value, &ParserContext::default()).unwrap();

        assert_str_eq!(
            info.pretty_print(),
            indoc! {r#"
            {
              "title": "foo",
              "license": {
                "name": "bar",
                "url": "baz"
              },
              "version": "1.0.0"
            }"# }
        );
    }

    #[test]
    fn fails_on_missing_title() {
        let value = json!({
          "version": "1.0.0"
        });

        let err = Info::try_parse(value, &ParserContext::default()).unwrap_err();

        assert_eq!(err.to_string(), "Info object should contain a title");
    }

    #[test]
    fn fails_on_missing_version() {
        let value = json!({
          "title": "foo"
        });

        let err = Info::try_parse(value, &ParserContext::default()).unwrap_err();

        assert_eq!(err.to_string(), "Info object should contain a version");
    }
}
