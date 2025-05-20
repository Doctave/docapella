pub mod r#box;
pub mod code_tabs;
pub mod flex;
pub mod grid;
// pub mod open_api;
pub mod steps;
pub mod tabs;

use std::collections::HashMap;

pub use code_tabs::CodeSelect;
pub use flex::Flex;
pub use grid::Grid;
pub use r#box::CBox;
pub use steps::{Step, Steps};
pub use tabs::{Tab, Tabs};

use crate::{
    content_ast::NodeKind as ContentNodeKind,
    markdown::custom_components::custom_component::{Error as ComponentError, Result},
    renderable_ast::Position,
    Attribute,
};

pub static EXPANDED_KEY: &str = "expanded";
pub static OPENAPI_PATH_KEY: &str = "openapi_path";

pub use self::{
    flex::{
        ALIGN_KEY, CLASS_KEY as FLEX_CLASS_KEY, DIRECTION_KEY, GAP_KEY,
        HEIGHT_KEY as FLEX_HEIGHT_KEY, JUSTIFY_KEY, WRAP_KEY,
    },
    grid::COLUMNS_KEY,
    r#box::{CLASS_KEY, HEIGHT_KEY, MAX_WIDTH_KEY, PADDING_KEY},
    tabs::TITLE_KEY,
};

pub enum Primitive {
    Tabs,
    Tab,
    Steps,
    Step,
    CodeSelect,
    Flex,
    Box,
    Grid,
    Slot,
    OpenAPISchema,
}

impl Primitive {
    pub fn parse_from_str(name: &str) -> Option<Primitive> {
        match name {
            "Tabs" => Some(Primitive::Tabs),
            "Tab" => Some(Primitive::Tab),
            "Steps" => Some(Primitive::Steps),
            "Step" => Some(Primitive::Step),
            "CodeSelect" => Some(Primitive::CodeSelect),
            "Flex" => Some(Primitive::Flex),
            "Box" => Some(Primitive::Box),
            "Grid" => Some(Primitive::Grid),
            "Slot" => Some(Primitive::Slot),
            "OpenAPISchema" => Some(Primitive::OpenAPISchema),
            _ => None,
        }
    }

    pub(crate) fn try_into_content_node_kind(
        self,
        attributes: Vec<Attribute>,
        node_pos: &Position,
    ) -> Result<ContentNodeKind> {
        let mut h = attributes.into_iter().fold(HashMap::new(), |mut a, next| {
            if let Some(val) = next.value {
                a.insert(next.key, val);
            }
            a
        });

        let attributes = match self {
            Primitive::Tabs => vec![],
            Primitive::Tab => vec![TITLE_KEY],
            Primitive::Steps => vec![],
            Primitive::Step => vec![TITLE_KEY],
            Primitive::CodeSelect => vec![TITLE_KEY],
            Primitive::Flex => vec![
                ALIGN_KEY,
                JUSTIFY_KEY,
                DIRECTION_KEY,
                WRAP_KEY,
                GAP_KEY,
                PADDING_KEY,
                FLEX_HEIGHT_KEY,
                FLEX_CLASS_KEY,
            ],
            Primitive::Box => vec![PADDING_KEY, CLASS_KEY, MAX_WIDTH_KEY, HEIGHT_KEY],
            Primitive::Grid => vec![COLUMNS_KEY, GAP_KEY],
            Primitive::Slot => vec![],
            Primitive::OpenAPISchema => vec![TITLE_KEY, EXPANDED_KEY, OPENAPI_PATH_KEY],
        };

        for incoming in h.keys() {
            if incoming == "if" || incoming == "elseif" || incoming == "else" {
                // these are special attributes
                continue;
            }

            if !attributes.contains(&incoming.as_str()) {
                // we have unexpected attributes
                return Err(ComponentError::UnexpectedAttribute(
                    incoming.clone(),
                    node_pos.clone(),
                ));
            }
        }

        let node_kind = match self {
            Primitive::Tabs => ContentNodeKind::Tabs,
            Primitive::Tab => ContentNodeKind::Tab {
                title: h.remove(TITLE_KEY),
            },
            Primitive::Steps => ContentNodeKind::Steps,
            Primitive::Step => ContentNodeKind::Step {
                title: h.remove(TITLE_KEY),
            },
            Primitive::CodeSelect => ContentNodeKind::CodeSelect {
                title: h.remove(TITLE_KEY),
            },
            Primitive::Flex => ContentNodeKind::Flex {
                align: h.remove(ALIGN_KEY),
                justify: h.remove(JUSTIFY_KEY),
                direction: h.remove(DIRECTION_KEY),
                wrap: h.remove(WRAP_KEY),
                gap: h.remove(GAP_KEY),
                padding: h.remove(PADDING_KEY),
                height: h.remove(FLEX_HEIGHT_KEY),
                class: h.remove(FLEX_CLASS_KEY),
            },
            Primitive::Box => ContentNodeKind::Box {
                padding: h.remove(PADDING_KEY),
                class: h.remove(CLASS_KEY),
                max_width: h.remove(MAX_WIDTH_KEY),
                height: h.remove(HEIGHT_KEY),
            },
            Primitive::Grid => ContentNodeKind::Grid {
                cols: h.remove(COLUMNS_KEY),
                gap: h.remove(GAP_KEY),
            },
            Primitive::Slot => ContentNodeKind::Slot,
            Primitive::OpenAPISchema => ContentNodeKind::OpenAPISchema {
                title: h.remove(TITLE_KEY),
                expanded: h.remove(EXPANDED_KEY),
                openapi_path: h.remove(OPENAPI_PATH_KEY),
            },
        };

        Ok(node_kind)
    }
}
