use located_yaml::{Marker, Yaml, YamlElt, YamlLoader};

use core::fmt;

use crate::{
    expressions::{self, Value},
    markdown::{
        custom_components::custom_component::{Error, Result},
        error_renderer::Location,
    },
};

#[derive(Deserialize, Debug, Clone)]
pub(crate) struct Attribute {
    pub title: String,
    pub default: Option<AttributeTypeValue>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub validation: Validation,
}

impl Attribute {
    pub fn verify(&self) -> Result<()> {
        for one_of in &self.validation.is_one_of {
            let matches = match self.validation.is_a {
                AttributeType::Number => matches!(one_of, AttributeTypeValue::Number(_)),
                AttributeType::Boolean => matches!(one_of, AttributeTypeValue::Boolean(_)),
                AttributeType::Text => matches!(one_of, AttributeTypeValue::Text(_)),
                AttributeType::Any => true,
            };

            if !matches {
                Err(Error::MismatchOneOf(
                    self.clone(),
                    self.validation.is_a.clone(),
                    one_of.clone(),
                ))?;
            }
        }

        if let Some(default_val) = &self.default {
            let matches = match self.validation.is_a {
                AttributeType::Number => matches!(default_val, AttributeTypeValue::Number(_)),
                AttributeType::Boolean => matches!(default_val, AttributeTypeValue::Boolean(_)),
                AttributeType::Text => matches!(default_val, AttributeTypeValue::Text(_)),
                AttributeType::Any => true,
            };

            if !matches {
                Err(Error::MismatchDefault(
                    self.clone(),
                    self.validation.is_a.clone(),
                    default_val.clone(),
                ))?;
            }

            if !self.validation.is_one_of.is_empty()
                && self
                    .validation
                    .is_one_of
                    .iter()
                    .all(|one| one.value_as_string() != default_val.value_as_string())
            {
                return Err(Error::InvalidDefault(
                    self.clone(),
                    default_val.value_as_string(),
                ));
            }
        }

        if !self
            .title
            .chars()
            .all(|c| c.is_ascii_alphabetic() || c == '_')
        {
            return Err(Error::InvalidTitle(
                self.clone(),
                "Title can only include alphabets and underscores".to_string(),
            ));
        }

        Ok(())
    }

    pub fn verify_incoming(&self, val: Value, expr: &str) -> expressions::Result<Value> {
        let type_problem = match self.validation.is_a {
            AttributeType::Text => !matches!(val, Value::String(_)),
            AttributeType::Number => !matches!(val, Value::Number(_)),
            AttributeType::Boolean => !matches!(val, Value::Bool(_)),
            AttributeType::Any => false,
        };

        if type_problem {
            let error = expressions::Error::UnexpectedType(
                val.to_string(),
                self.validation.is_a.to_string(),
                expressions::Span {
                    start: 0,
                    end: expr.to_string().len(),
                },
            );

            Err(error)?;
        }

        if !self.validation.is_one_of.is_empty() {
            let enum_problem = !self
                .validation
                .is_one_of
                .iter()
                .map(|v| v.value_as_string())
                .collect::<Vec<String>>()
                .contains(&val.to_string());

            if enum_problem {
                let error = expressions::Error::UnexpectedEnum(
                    val.to_string(),
                    self.is_one_of_string(),
                    expressions::Span {
                        start: 0,
                        end: expr.to_string().len(),
                    },
                );

                Err(error)?;
            }
        }

        Ok(val)
    }

    pub fn locate_from_source(&self, source: &str) -> Option<LocatedAttribute> {
        let mut res = YamlLoader::load_from_str(source).unwrap();
        let mut located = None;

        let root = res
            .docs
            .get_mut(0)
            .expect("This is a bug. Attributes didn't get validated.");

        // find our attribute
        if let Some(YamlElt::Array(attributes)) = root.get_string_key("attributes").map(|n| n.yaml)
        {
            located = attributes.into_iter().find(|a| {
                if let Some(title) = a.get_string_key("title") {
                    let title = match title.yaml {
                        YamlElt::Real(r) => r.to_string(),
                        YamlElt::String(s) => s,
                        YamlElt::Integer(i) => i.to_string(),
                        YamlElt::Boolean(b) => b.to_string(),
                        _ => {
                            return false;
                        }
                    };
                    return title == self.title;
                }

                false
            });
        }

        located.map(|l| LocatedAttribute {
            attribute: self.clone(),
            located: l.clone(),
        })
    }

    pub fn is_one_of_string(&self) -> String {
        self.validation
            .is_one_of
            .iter()
            .map(|v| v.value_as_string())
            .collect::<Vec<String>>()
            .join(", ")
    }
}

#[derive(Deserialize, Debug, Clone, Default)]
pub(crate) struct Validation {
    #[serde(default)]
    pub is_a: AttributeType,
    #[serde(default)]
    pub is_one_of: Vec<AttributeTypeValue>,
}

#[derive(Debug, Clone)]
pub struct LocatedAttribute {
    attribute: Attribute,
    located: Yaml,
}

impl LocatedAttribute {
    pub fn title_location(&self) -> Option<(Location, Location)> {
        self.locate("title", &self.attribute.title)
    }

    pub fn default_location(&self) -> Option<(Location, Location)> {
        self.attribute
            .default
            .as_ref()
            .and_then(|default| self.locate("default", &default.value_as_string()))
    }

    pub fn is_a_location(&self) -> Option<(Location, Location)> {
        self.locate("is_a", &self.attribute.validation.is_a.to_string())
    }

    pub fn is_one_of_location(&self, value: &str) -> Option<(Location, Location)> {
        self.locate("is_one_of", value)
    }

    fn locate(&self, key: &str, value: &str) -> Option<(Location, Location)> {
        Self::find_key_value_location(&self.located, &key.to_string(), &value.to_string()).map(
            |(key_mark, value_mark)| {
                (
                    Location::Point(key_mark.line, key_mark.col + 1),
                    Location::Point(value_mark.line, value_mark.col + 1),
                )
            },
        )
    }

    fn find_matching_value<'a>(expected_value: &String, value: &'a Yaml) -> Option<&'a Yaml> {
        let evaluate_value = |some: &String| {
            if &some.to_string() == expected_value {
                Some(value)
            } else {
                None
            }
        };

        match &value.yaml {
            YamlElt::Real(r) => return evaluate_value(r),
            YamlElt::String(s) => return evaluate_value(s),
            YamlElt::Integer(i) => return evaluate_value(&i.to_string()),
            YamlElt::Boolean(b) => return evaluate_value(&b.to_string()),
            YamlElt::Array(a) => {
                for arr_value in a {
                    let res = Self::find_matching_value(expected_value, arr_value);

                    if res.is_some() {
                        return res;
                    }
                }
            }
            _ => {}
        }

        None
    }

    fn find_key_value_location(
        current_value: &Yaml,
        key: &String,
        value: &String,
    ) -> Option<(Marker, Marker)> {
        if let YamlElt::Hash(hash) = &current_value.yaml {
            for (k, v) in hash {
                if matches!(&v.yaml, YamlElt::Hash(_)) {
                    return Self::find_key_value_location(v, key, value);
                } else if Self::find_matching_value(key, k).is_some() {
                    if let Some(val) = Self::find_matching_value(value, v) {
                        return Some((k.marker, val.marker));
                    }
                }
            }
        };

        None
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(untagged)]
pub enum AttributeTypeValue {
    Text(String),
    Number(isize),
    Boolean(bool),
}

impl AttributeTypeValue {
    pub fn value_as_string(&self) -> String {
        match self {
            AttributeTypeValue::Boolean(val) => val.to_string(),
            AttributeTypeValue::Number(val) => val.to_string(),
            AttributeTypeValue::Text(val) => val.to_string(),
        }
    }

    pub fn as_string(&self) -> String {
        match self {
            AttributeTypeValue::Boolean(val) => val.to_string(),
            AttributeTypeValue::Number(val) => val.to_string(),
            AttributeTypeValue::Text(val) => val.to_string(),
        }
    }
}

#[derive(Deserialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AttributeType {
    Any,
    Text,
    Number,
    Boolean,
}

impl Default for AttributeType {
    fn default() -> Self {
        Self::Any
    }
}

impl From<AttributeTypeValue> for AttributeType {
    fn from(value: AttributeTypeValue) -> Self {
        match value {
            AttributeTypeValue::Boolean(_) => AttributeType::Boolean,
            AttributeTypeValue::Number(_) => AttributeType::Number,
            AttributeTypeValue::Text(_) => AttributeType::Text,
        }
    }
}

impl fmt::Display for AttributeTypeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttributeTypeValue::Boolean(_) => f.write_str("boolean"),
            AttributeTypeValue::Number(_) => f.write_str("number"),
            AttributeTypeValue::Text(_) => f.write_str("text"),
        }
    }
}

impl fmt::Display for AttributeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Boolean => f.write_str("boolean"),
            Self::Text => f.write_str("text"),
            Self::Number => f.write_str("number"),
            Self::Any => f.write_str("any"),
        }
    }
}
