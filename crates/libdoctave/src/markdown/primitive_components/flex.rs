use crate::{
    autocomplete::PrimitiveComponentAutocomplete,
    expressions::Value,
    markdown::error_renderer::{self, parse_int_in_range, Highlight, Location},
    render_context::RenderContext,
    renderable_ast::Position,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

// Keys when we deserialize AST attributes
pub static ALIGN_KEY: &str = "align";
pub static PADDING_KEY: &str = "pad";
pub static GAP_KEY: &str = "gap";
pub static WRAP_KEY: &str = "wrap";
pub static JUSTIFY_KEY: &str = "justify";
pub static DIRECTION_KEY: &str = "dir";
pub static HEIGHT_KEY: &str = "height";
pub static CLASS_KEY: &str = "class";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub struct Flex {
    pub align: FlexAlign,
    pub padding: usize,
    pub direction: FlexDirection,
    pub justify: FlexJustify,
    pub gap: usize,
    pub wrap: FlexWrap,
    pub height: FlexHeight,
    pub class: String,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid {JUSTIFY_KEY}. Expected one of `start`, `center`, `end`, or `between`."#)]
    InvalidJustify(String),
    #[error(
        r#"Invalid {ALIGN_KEY}. Expected one of `start`, `center`, `end`, `baseline`, or `stretch`."#
    )]
    InvalidAlign(String),
    #[error(r#"Invalid {DIRECTION_KEY}. Expected one of `row`, `column`, `row-reverse`, or `column-reverse`."#)]
    InvalidDirection(String),
    #[error(r#"Invalid {WRAP_KEY}. Expected one of `wrap`, `nowrap`, or `wrap-reverse`"#)]
    InvalidWrap(String),
    #[error(r#"Invalid {GAP_KEY}. Expected value to be a number between 0 and 5."#)]
    InvalidGap(String),
    #[error(r#"Invalid {PADDING_KEY}. Expected value to be a number between 0 and 5."#)]
    InvalidPadding(String),
    #[error(r#"Invalid {HEIGHT_KEY}. Expected `auto` or `full`."#)]
    InvalidHeight(String),
    #[error(r#"Missing required attribute {0}"#)]
    MissingAttribute(&'static str),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, node_pos: &Position) -> String {
        let mut highlights = vec![];

        match self {
            Error::InvalidJustify(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, JUSTIFY_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::InvalidGap(found) => {
                let pos = error_renderer::offset_attribute_error_pos(md, GAP_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::InvalidPadding(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, PADDING_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::InvalidAlign(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, ALIGN_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::InvalidDirection(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, DIRECTION_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::InvalidWrap(found) => {
                let pos = error_renderer::offset_attribute_error_pos(md, WRAP_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            Error::InvalidHeight(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, HEIGHT_KEY, found, node_pos);
                let location = Location::Point(pos.start.row, pos.start.col + 1);

                let highlight = Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                };

                highlights.push(highlight);
            }
            _ => {}
        }

        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }
}

impl Flex {
    #[allow(clippy::too_many_arguments)]
    pub fn try_new(
        justify: Option<Value>,
        align: Option<Value>,
        direction: Option<Value>,
        wrap: Option<Value>,
        gap: Option<Value>,
        padding: Option<Value>,
        height: Option<Value>,
        class: Option<Value>,
    ) -> Result<Self> {
        let mut flex = Self::default();
        if let Some(align) = align {
            flex.align = align.to_string().as_str().try_into()?;
        }
        if let Some(justify) = justify {
            flex.justify = justify.to_string().as_str().try_into()?;
        }
        if let Some(direction) = direction {
            flex.direction = direction.to_string().as_str().try_into()?;
        }
        if let Some(wrap) = wrap {
            flex.wrap = wrap.to_string().as_str().try_into()?;
        }
        if let Some(gap) = gap {
            flex.gap = parse_int_in_range(gap.to_string().as_str(), 0..6)
                .ok_or(Error::InvalidGap(gap.to_string()))?;
        }
        if let Some(padding) = padding {
            flex.padding = parse_int_in_range(padding.to_string().as_str(), 0..6)
                .ok_or(Error::InvalidPadding(padding.to_string()))?;
        }
        if let Some(height) = height {
            flex.height = height.to_string().as_str().try_into()?;
        }
        if let Some(class) = class {
            flex.class = class.to_string();
        }

        Ok(flex)
    }
}

impl PrimitiveComponentAutocomplete for Flex {
    fn title(&self) -> &str {
        "Flex"
    }

    fn attributes(&self) -> Vec<&str> {
        vec![
            "justify",
            "align",
            "direction",
            "wrap",
            "gap",
            "padding",
            "height",
            "class",
        ]
    }

    fn attribute_values(&self, attribute: &str) -> Vec<&str> {
        match attribute {
            "justify" => vec!["start", "end", "center", "between", "around"],
            "align" => vec!["start", "end", "center", "stretch"],
            "direction" => vec!["row", "column"],
            "wrap" => vec!["nowrap", "wrap", "wrap-reverse"],
            "gap" => vec!["0", "1", "2", "3", "4", "5"],
            "padding" => vec!["0", "1", "2", "3", "4", "5"],
            "height" => vec!["auto", "full"],
            _ => vec![],
        }
    }
}

impl Default for Flex {
    fn default() -> Self {
        Self {
            justify: FlexJustify::Start,
            align: FlexAlign::Start,
            direction: FlexDirection::Row,
            wrap: FlexWrap::Nowrap,
            gap: 0,
            padding: 0,
            height: FlexHeight::Auto,
            class: String::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexAlign {
    Start,
    Center,
    End,
    Baseline,
    Stretch,
}

impl TryFrom<&str> for FlexAlign {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "start" => Ok(FlexAlign::Start),
            "center" => Ok(FlexAlign::Center),
            "end" => Ok(FlexAlign::End),
            "baseline" => Ok(FlexAlign::Baseline),
            "stretch" => Ok(FlexAlign::Stretch),
            _ => Err(Error::InvalidAlign(value.to_string())),
        }
    }
}
// format!("Unexpected value `{}` for attribute `align`.\nExpected one of `start`, `center`, `end`, `baseline`, or `stretch`.", value)

#[derive(Debug, Default, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexJustify {
    #[default]
    Start,
    Center,
    End,
    Between,
}

impl TryFrom<&str> for FlexJustify {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "start" => Ok(FlexJustify::Start),
            "center" => Ok(FlexJustify::Center),
            "end" => Ok(FlexJustify::End),
            "between" => Ok(FlexJustify::Between),
            _ => Err(Error::InvalidJustify(value.to_string())),
        }
    }
}
// "Unexpected value `{}` for attribute `justify`.\nExpected one of `start`, `center`, `end`, or `between`."

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexDirection {
    Row,
    Column,
    RowReverse,
    ColumnReverse,
}

impl TryFrom<&str> for FlexDirection {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "row" => Ok(FlexDirection::Row),
            "column" => Ok(FlexDirection::Column),
            "row-reverse" => Ok(FlexDirection::RowReverse),
            "column-reverse" => Ok(FlexDirection::ColumnReverse),
            _ => Err(Error::InvalidDirection(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexWrap {
    Wrap,
    Nowrap,
    WrapReverse,
}

impl TryFrom<&str> for FlexWrap {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "wrap" => Ok(FlexWrap::Wrap),
            "nowrap" => Ok(FlexWrap::Nowrap),
            "wrap-reverse" => Ok(FlexWrap::WrapReverse),
            _ => Err(Error::InvalidWrap(value.to_string())),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FlexHeight {
    Auto,
    Full,
}

impl TryFrom<&str> for FlexHeight {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "auto" => Ok(FlexHeight::Auto),
            "full" => Ok(FlexHeight::Full),
            _ => Err(Error::InvalidHeight(value.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast_mdx;
    use crate::render_context::RenderContext;

    mod flex {

        use pretty_assertions::assert_str_eq;

        use super::*;

        #[test]
        fn basic() {
            let markdown = indoc! {r#"
            <Flex>
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn unexpected_attributes() {
            let markdown = indoc! {r#"
            <Flex foobar="booboo">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Unexpected attribute");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Unexpected attribute "foobar"

                    1 │ <Flex foobar="booboo">
                              ▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn justify() {
            let markdown = indoc! {r#"
            <Flex justify="center">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Center} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_justify() {
            let markdown = indoc! {r#"
            <Flex justify="booboo">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! { r#"
                Invalid justify. Expected one of `start`, `center`, `end`, or `between`.

                    1 │ <Flex justify="booboo">
                                       ▲▲▲▲▲▲

                "# }
            );
        }

        #[test]
        fn align() {
            let markdown = indoc! {r#"
            <Flex align="center">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Center} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_align() {
            let markdown = indoc! {r#"
            <Flex align="booboo">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid align. Expected one of `start`, `center`, `end`, `baseline`, or `stretch`.

                    1 │ <Flex align="booboo">
                                     ▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn direction() {
            let markdown = indoc! {r#"
            <Flex dir="column-reverse">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={ColumnReverse} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_direction() {
            let markdown = indoc! {r#"
            <Flex dir="asdfasdf">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid dir. Expected one of `row`, `column`, `row-reverse`, or `column-reverse`.

                    1 │ <Flex dir="asdfasdf">
                                   ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn wrap() {
            let markdown = indoc! {r#"
            <Flex wrap="nowrap">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_wrap() {
            let markdown = indoc! {r#"
            <Flex wrap="asdfasdf">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid wrap. Expected one of `wrap`, `nowrap`, or `wrap-reverse`

                    1 │ <Flex wrap="asdfasdf">
                                    ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn gap() {
            let markdown = indoc! {r#"
            <Flex gap="4">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={4} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_gap() {
            let markdown = indoc! {r#"
            <Flex gap="asdfasdf">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid gap. Expected value to be a number between 0 and 5.

                    1 │ <Flex gap="asdfasdf">
                                   ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn padding() {
            let markdown = indoc! {r#"
            <Flex pad="4">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={4} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_padding() {
            let markdown = indoc! {r#"
            <Flex pad="asdfasdf">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid pad. Expected value to be a number between 0 and 5.

                    1 │ <Flex pad="asdfasdf">
                                   ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn height() {
            let markdown = indoc! {r#"
            <Flex height="auto">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn full_height() {
            let markdown = indoc! {r#"
            <Flex height="full">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Full} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn default_auto_height() {
            let markdown = indoc! {r#"
            <Flex>
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn invalid_height() {
            let markdown = indoc! {r#"
            <Flex height="asdfasdf">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(error.message, "Error in flex");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid height. Expected `auto` or `full`.

                    1 │ <Flex height="asdfasdf">
                                      ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn flex_wrap() {
            let markdown = indoc! {r#"
            <Flex wrap="wrap">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Wrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn class() {
            let markdown = indoc! {r#"
            <Flex class="custom">
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={custom}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn class_default_none() {
            let markdown = indoc! {r#"
            <Flex>
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={}>
                </Flex>
                "# },
            );
        }

        #[test]
        fn class_can_be_anything() {
            let markdown = indoc! {r#"
            <Flex class={123}>
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Flex justify={Start} align={Start} direction={Row} wrap={Nowrap} gap={0} padding={0} height={Auto} class={123}>
                </Flex>
                "# },
            );
        }
    }
}
