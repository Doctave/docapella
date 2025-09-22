use serde::Serialize;
use thiserror::Error;

use crate::{
    autocomplete::PrimitiveComponentAutocomplete,
    expressions::Value,
    markdown::error_renderer::{self, parse_int_in_range, Highlight, Location},
    render_context::RenderContext,
    renderable_ast::Position,
};

pub type Result<T> = std::result::Result<T, Error>;

pub static MAX_WIDTH_KEY: &str = "max_width";
pub static PADDING_KEY: &str = "pad";
pub static CLASS_KEY: &str = "class";
pub static HEIGHT_KEY: &str = "height";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
#[serde(rename = "Box")]
pub struct CBox {
    pub padding: usize,
    pub max_width: MaxWidth,
    pub class: String,
    pub height: Height,
}

impl CBox {
    pub fn new(padding: usize, max_width: MaxWidth, class: String, height: Height) -> Self {
        CBox {
            padding,
            max_width,
            class,
            height,
        }
    }

    pub fn try_new(
        max_width: Option<Value>,
        class: Option<Value>,
        padding: Option<Value>,
        height: Option<Value>,
    ) -> Result<Self> {
        let mut c_box = Self::default();

        if let Some(max_width) = max_width {
            c_box.max_width = max_width.to_string().as_str().try_into()?;
        }
        if let Some(padding) = padding {
            c_box.padding = parse_int_in_range(padding.to_string().as_str(), 0..6)
                .ok_or(Error::InvalidPadding(padding.to_string()))?;
        }
        if let Some(class) = class {
            c_box.class = class.to_string();
        }
        if let Some(height) = height {
            c_box.height = height.to_string().as_str().try_into()?;
        }

        Ok(c_box)
    }
}

impl Default for CBox {
    fn default() -> Self {
        CBox {
            padding: 0,
            max_width: MaxWidth::Full,
            class: "".to_string(),
            height: Height::Auto,
        }
    }
}

impl PrimitiveComponentAutocomplete for CBox {
    fn title(&self) -> &str {
        "Box"
    }

    fn attributes(&self) -> Vec<&str> {
        vec!["max_width", "pad", "class", "height"]
    }

    fn attribute_values(&self, attribute: &str) -> Vec<&str> {
        match attribute {
            "max_width" => vec!["sm", "md", "lg", "xl", "full"],
            "pad" => vec!["0", "1", "2", "3", "4", "5"],
            "height" => vec!["auto", "full"],
            _ => vec![],
        }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid {MAX_WIDTH_KEY}. Expected one of `sm`, `ms`, `lg`, `xl` or `full`."#)]
    InvalidMaxWidth(String),
    #[error(r#"Invalid {PADDING_KEY}. Expected value to be a number between 0 and 5."#)]
    InvalidPadding(String),
    #[error(r#"Invalid {HEIGHT_KEY}. Expected `auto` or `full`."#)]
    InvalidHeight(String),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, node_pos: &Position) -> String {
        let mut highlights = vec![];

        match self {
            Error::InvalidMaxWidth(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, MAX_WIDTH_KEY, found, node_pos);
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
        }

        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum MaxWidth {
    Sm,
    Md,
    Lg,
    Xl,
    Full,
}

impl std::fmt::Display for MaxWidth {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use MaxWidth::*;
        match self {
            Sm => write!(f, "sm"),
            Md => write!(f, "md"),
            Lg => write!(f, "lg"),
            Xl => write!(f, "xl"),
            Full => write!(f, "full"),
        }
    }
}

impl TryFrom<&str> for MaxWidth {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "sm" => Ok(MaxWidth::Sm),
            "md" => Ok(MaxWidth::Md),
            "lg" => Ok(MaxWidth::Lg),
            "xl" => Ok(MaxWidth::Xl),
            "full" => Ok(MaxWidth::Full),
            _ => Err(Error::InvalidMaxWidth(value.to_string())),
        }
    }
    // format!("Unexpected value `{}` for attribute `max_width`.\nExpected one of `sm`, `ms`, `lg`, `xl` or `full`.", value)
}

#[derive(Debug, Clone, Copy, PartialEq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Height {
    Auto,
    Full,
}

impl TryFrom<&str> for Height {
    type Error = Error;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        match value {
            "auto" => Ok(Height::Auto),
            "full" => Ok(Height::Full),
            _ => Err(Error::InvalidHeight(value.to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast_mdx;
    use crate::render_context::RenderContext;
    mod box_component {
        use pretty_assertions::assert_str_eq;

        use super::*;
        #[test]
        fn basic() {
            let markdown = indoc! {r#"
              <Box>
              </Box>
              "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Box padding={0} max_width={full} class={} height={Auto}>
                </Box>
                "# }
            );
        }

        #[test]
        fn unexpected_attributes() {
            let markdown = indoc! {r#"
            <Box foobar="booboo">
            </Box>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Unexpected attribute");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Unexpected attribute "foobar"

                    1 │ <Box foobar="booboo">
                             ▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn padding() {
            let markdown = indoc! {r#"
              <Box pad="4">
              </Box>
              "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Box padding={4} max_width={full} class={} height={Auto}>
                </Box>
                "# }
            );
        }

        #[test]
        fn invalid_padding() {
            let markdown = indoc! {r#"
              <Box pad="asdfasdf">
              </Box>
              "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Error in box");
            assert_str_eq!(
                error.description,
                indoc! { r#"
                Invalid pad. Expected value to be a number between 0 and 5.

                    1 │ <Box pad="asdfasdf">
                                  ▲▲▲▲▲▲▲▲

              "# }
            )
        }

        #[test]
        fn max_width() {
            let markdown = indoc! {r#"
              <Box max_width="sm">
              </Box>
              "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                  <Box padding={0} max_width={sm} class={} height={Auto}>
                  </Box>
                  "# }
            );
        }

        #[test]
        fn invalid_max_width() {
            let markdown = indoc! {r#"
              <Box max_width="foobar">
              </Box>
              "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Error in box");
            assert_str_eq!(
                error.description,
                indoc! { r#"
                Invalid max_width. Expected one of `sm`, `ms`, `lg`, `xl` or `full`.

                    1 │ <Box max_width="foobar">
                                        ▲▲▲▲▲▲

                "# }
            )
        }

        #[test]
        fn class() {
            let markdown = indoc! {r#"
              <Box class="something">
              </Box>
              "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                  <Box padding={0} max_width={full} class={something} height={Auto}>
                  </Box>
                  "# }
            );
        }

        #[test]
        fn class_can_be_anything() {
            let markdown = indoc! {r#"
              <Box class={123}>
              </Box>
              "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                  <Box padding={0} max_width={full} class={123} height={Auto}>
                  </Box>
                  "# }
            );
        }

        #[test]
        fn height() {
            let markdown = indoc! {r#"
            <Box height="auto">
            </Box>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Box padding={0} max_width={full} class={} height={Auto}>
                </Box>
                "# },
            );
        }

        #[test]
        fn full_height() {
            let markdown = indoc! {r#"
            <Box height="full">
            </Box>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Box padding={0} max_width={full} class={} height={Full}>
                </Box>
                "# },
            );
        }

        #[test]
        fn default_auto_height() {
            let markdown = indoc! {r#"
            <Box>
            </Box>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Box padding={0} max_width={full} class={} height={Auto}>
                </Box>
                "# },
            );
        }

        #[test]
        fn invalid_height() {
            let markdown = indoc! {r#"
            <Box height="asdfasdf">
            </Box>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(error.message, "Error in box");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid height. Expected `auto` or `full`.

                    1 │ <Box height="asdfasdf">
                                     ▲▲▲▲▲▲▲▲

                "#}
            );
        }
    }
}
