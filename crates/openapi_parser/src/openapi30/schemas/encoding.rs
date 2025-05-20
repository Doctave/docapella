use super::header::Header;
use crate::openapi30::parser;
use crate::openapi30::parser::ParserContext;
use crate::Map;
use crate::Set;
use crate::String;
use crate::Value;

#[cfg(test)]
use serde_json::to_string_pretty;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Failed to parse header: {0}"#)]
    HeaderParseError(String),
}

#[derive(Debug, Clone)]
pub struct Encoding {
    pub content_type: Option<String>,
    pub headers: Map<String, Header>,
    pub style: Option<String>,
    pub explode: Option<bool>,
    pub allow_reserved: Option<bool>,
}

impl Encoding {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        mut value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let content_type = value.take("contentType").and_then(Value::take_string);

        let headers = value
            .take("headers")
            .and_then(Value::take_object)
            .map(|obj| {
                obj.into_iter()
                    .map(|(k, v)| {
                        Header::try_parse(v, k.clone(), ctx, visited_refs).map(|h| (k, h))
                    })
                    .collect::<Result<Map<_, _>, _>>()
            })
            .transpose()?
            .unwrap_or_default();

        let style = value.take("style").and_then(Value::take_string);
        let explode = value.take("explode").and_then(Value::take_bool);
        let allow_reserved = value.take("allowReserved").and_then(Value::take_bool);

        Ok(Encoding {
            content_type,
            headers,
            style,
            explode,
            allow_reserved,
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
    fn parses_content_type() {
        let value = json!({
            "contentType": "application/json"
        });
        let encoding =
            Encoding::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            encoding.pretty_print(),
            indoc! {r#"
            {
              "contentType": "application/json"
            }"#}
        );
    }

    #[test]
    fn parses_headers() {
        let value = json!({
            "headers": {
                "X-Custom-Header": {
                    "description": "A custom header"
                }
            }
        });
        let encoding =
            Encoding::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            encoding.pretty_print(),
            indoc! {r#"
            {
              "headers": {
                "X-Custom-Header": {
                  "name": "X-Custom-Header",
                  "in": "header",
                  "description": "A custom header"
                }
              }
            }"#}
        );
    }

    #[test]
    fn parses_style() {
        let value = json!({
            "style": "form"
        });
        let encoding =
            Encoding::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            encoding.pretty_print(),
            indoc! {r#"
            {
              "style": "form"
            }"#}
        );
    }

    #[test]
    fn parses_explode() {
        let value = json!({
            "explode": true
        });
        let encoding =
            Encoding::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            encoding.pretty_print(),
            indoc! {r#"
            {
              "explode": true
            }"#}
        );
    }

    #[test]
    fn parses_allow_reserved() {
        let value = json!({
            "allowReserved": true
        });
        let encoding =
            Encoding::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            encoding.pretty_print(),
            indoc! {r#"
            {
              "allowReserved": true
            }"#}
        );
    }

    #[test]
    fn parses_all_fields() {
        let value = json!({
            "contentType": "application/json",
            "headers": {
                "X-Custom-Header": {
                    "description": "A custom header"
                }
            },
            "style": "form",
            "explode": true,
            "allowReserved": false
        });
        let encoding =
            Encoding::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            encoding.pretty_print(),
            indoc! {r#"
            {
              "contentType": "application/json",
              "headers": {
                "X-Custom-Header": {
                  "name": "X-Custom-Header",
                  "in": "header",
                  "description": "A custom header"
                }
              },
              "style": "form",
              "explode": true,
              "allowReserved": false
            }"#}
        );
    }
}
