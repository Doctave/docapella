use crate::openapi30::parser;
use crate::String;
use crate::Value;
#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"License object should contain a name"#)]
    MissingName,
}

#[derive(Debug, Clone)]
pub struct License {
    pub name: String,
    pub url: Option<String>,
}

impl License {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(mut value: Value) -> parser::Result<Self> {
        let name = value
            .take("name")
            .and_then(Value::take_string)
            .ok_or(Error::MissingName)?;

        let url = value.take("url").and_then(Value::take_string);

        Ok(License { name, url })
    }
}

#[cfg(test)]
mod test {
    use crate::{json, openapi30::schemas::license::License};
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn parses_license() {
        let value = json!({
            "name": "Apache 2.0"
        });
        let license = License::try_parse(value).unwrap();
        assert_str_eq!(
            license.pretty_print(),
            indoc! {r#"
            {
              "name": "Apache 2.0"
            }"#}
        )
    }

    #[test]
    fn parses_license_with_url() {
        let value = json!({
            "name": "Apache 2.0",
            "url": "https://www.apache.org/licenses/LICENSE-2.0.html"
        });
        let license = License::try_parse(value).unwrap();
        assert_str_eq!(
            license.pretty_print(),
            indoc! {r#"
            {
              "name": "Apache 2.0",
              "url": "https://www.apache.org/licenses/LICENSE-2.0.html"
            }"#}
        )
    }

    #[test]
    fn missing_name() {
        let value = json!({
            "url": "https://www.apache.org/licenses/LICENSE-2.0.html"
        });
        let err = License::try_parse(value).unwrap_err();
        assert_str_eq!(err.to_string(), r#"License object should contain a name"#)
    }
}
