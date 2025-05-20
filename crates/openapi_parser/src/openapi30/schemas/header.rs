use super::parameter::Parameter;
use crate::{
    openapi30::parser::{self, ParserContext},
    Set, String, Value,
};
#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Clone)]
pub struct Header(pub Parameter);

impl Header {
    #[cfg(test)]
    pub fn pretty_print(&self) -> String {
        let val: Value = self.clone().into();
        to_string_pretty(&val).unwrap().into()
    }

    pub fn try_parse(
        value: Value,
        name: String,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let mut visited_refs = visited_refs.clone();
        let mut value = value
            .get("$ref")
            .and_then(Value::as_str)
            .and_then(|ref_path| ctx.get(ref_path, &mut visited_refs))
            .unwrap_or(value);

        value.insert("name".into(), name.into());
        value.insert("in".into(), Value::String("header".into()));

        Parameter::try_parse(value, ctx, &mut visited_refs).map(Header)
    }
}

#[cfg(test)]
mod test {
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use super::*;

    #[test]
    fn sets_name_and_in() {
        let value = json!({
            "description": "Header description",
            "required": true,
            "schema": {
                "type": "string"
            }
        });
        let header = Header::try_parse(
            value,
            "X-Header".into(),
            &ParserContext::default(),
            &mut Set::new(),
        )
        .unwrap();

        assert_str_eq!(
            header.pretty_print(),
            indoc! {r#"
            {
              "name": "X-Header",
              "in": "header",
              "schema": {
                "description": "Header description",
                "type": "string",
                "doctave_metadata": {
                  "field_name": "X-Header"
                }
              },
              "description": "Header description",
              "required": true
            }"#}
        );
    }

    #[test]
    fn parses_from_ref_cache() {
        let value = json!({
            "$ref": "#/components/headers/Header"
        });

        let ref_cache = json!({
          "headers": {
            "Header": {
                "description": "Header description",
                "required": true,
                "schema": {
                    "type": "string"
                }
            }
          }
        });

        let mut ctx = ParserContext::default();

        ctx.ref_cache.with_value(ref_cache);

        let header = Header::try_parse(value, "X-Header".into(), &ctx, &mut Set::new()).unwrap();

        assert_str_eq!(
            header.pretty_print(),
            indoc! {r#"
            {
              "name": "X-Header",
              "in": "header",
              "schema": {
                "description": "Header description",
                "type": "string",
                "doctave_metadata": {
                  "field_name": "X-Header"
                }
              },
              "description": "Header description",
              "required": true
            }"#}
        );
    }
}
