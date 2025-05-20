use crate::{
    openapi30::parser::{self, ParserContext},
    String, Value,
};

use crate::Set;
#[cfg(test)]
use serde_json::to_string_pretty;

use super::Schema;

#[derive(Debug, Clone, PartialEq)]
pub struct AnyOfSchema {
    pub any_of: Vec<Schema>,
}

impl AnyOfSchema {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.clone().into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn is_empty(&self) -> bool {
        self.any_of.is_empty()
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.any_of.extend(other.any_of);
        self.any_of.dedup();

        self
    }

    pub fn try_parse(
        value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
    ) -> parser::Result<Self> {
        let any_of = value
            .take_array()
            .unwrap_or_default()
            .into_iter()
            .map(|v| Schema::try_parse(v, ctx, visited_refs, None))
            .collect::<Result<Vec<_>, _>>()?;

        Ok(AnyOfSchema { any_of })
    }
}

#[cfg(test)]
mod test {
    use crate::json;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    use super::*;

    #[test]
    fn parses_any_of_schema() {
        let value = json!([
            {"type": "string"},
            {"type": "number"}
        ]);
        let any_of_schema =
            AnyOfSchema::try_parse(value, &ParserContext::default(), &mut Set::new()).unwrap();
        assert_str_eq!(
            any_of_schema.pretty_print(),
            indoc! {r#"
            {
              "anyOf": [
                {
                  "type": "string"
                },
                {
                  "type": "number"
                }
              ]
            }"#}
        );
    }
}
