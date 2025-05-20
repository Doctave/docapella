use crate::{
    openapi30::parser::{self, ParserContext},
    Set, String, Value,
};

use super::Schema;

#[cfg(test)]
use serde_json::to_string_pretty;

#[derive(Debug, Clone, PartialEq)]
pub struct Property {
    pub required: bool,
    pub schema: Schema,
}

impl Property {
    #[cfg(test)]
    pub fn pretty_print(self) -> String {
        let val: Value = self.clone().into();

        to_string_pretty(&val).unwrap().into()
    }

    pub fn is_empty(&self) -> bool {
        self.schema.is_empty()
    }

    pub fn exclude(mut self, other: &mut Self) -> Self {
        self.schema.not = other.schema.not.take();
        self.schema.exclude();

        self
    }

    pub fn merge(mut self, other: Self) -> Self {
        self.required = other.required;

        let schema = self.schema.merge(other.schema);

        Property {
            required: self.required,
            schema,
        }
    }

    pub(crate) fn try_parse(
        value: Value,
        ctx: &ParserContext,
        visited_refs: &mut Set<String>,
        required: bool,
    ) -> parser::Result<Self> {
        let schema = Schema::try_parse(value, ctx, visited_refs, None)?;

        Ok(Property { required, schema })
    }
}
