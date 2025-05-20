use crate::openapi30::parser;
use crate::String;
use crate::Value;
use thiserror::Error;

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Contact object should contain at least one of name, url, or email"#)]
    EmptyContact,
}

#[derive(Debug, Clone)]
pub struct Contact {
    pub name: Option<String>,
    pub url: Option<String>,
    pub email: Option<String>,
}

impl Contact {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let name = value.take("name").and_then(Value::take_string);
        let url = value.take("url").and_then(Value::take_string);
        let email = value.take("email").and_then(Value::take_string);

        if name.is_none() && url.is_none() && email.is_none() {
            return Err(Error::EmptyContact.into());
        }

        Ok(Contact { name, url, email })
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
            "name": "API Support"
        });
        let contact = Contact::try_parse(value).unwrap();
        assert_str_eq!(
            contact.pretty_print(),
            indoc! {r#"
            {
              "name": "API Support"
            }"#}
        );
    }

    #[test]
    fn parses_url() {
        let value = json!({
            "url": "https://www.example.com/support"
        });
        let contact = Contact::try_parse(value).unwrap();
        assert_str_eq!(
            contact.pretty_print(),
            indoc! {r#"
            {
              "url": "https://www.example.com/support"
            }"#}
        );
    }

    #[test]
    fn parses_email() {
        let value = json!({
            "email": "support@example.com"
        });
        let contact = Contact::try_parse(value).unwrap();
        assert_str_eq!(
            contact.pretty_print(),
            indoc! {r#"
            {
              "email": "support@example.com"
            }"#}
        );
    }

    #[test]
    fn parses_contact() {
        let value = json!({
            "name": "API Support",
            "url": "https://www.example.com/support",
            "email": "support@example.com"
        });
        let contact = Contact::try_parse(value).unwrap();
        assert_str_eq!(
            contact.pretty_print(),
            indoc! {r#"
            {
              "name": "API Support",
              "url": "https://www.example.com/support",
              "email": "support@example.com"
            }"#}
        );
    }

    #[test]
    fn fails_on_empty_contact() {
        let value = json!({});
        let err = Contact::try_parse(value).unwrap_err();
        assert_str_eq!(
            err.to_string(),
            r#"Contact object should contain at least one of name, url, or email"#
        );
    }
}
