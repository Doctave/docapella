use serde::Serialize;
use thiserror::Error;

#[cfg(test)]
use ts_rs::TS;

#[cfg(feature = "rustler")]
use rustler::NifStruct;

use crate::{
    autocomplete::PrimitiveComponentAutocomplete,
    expressions::Value,
    markdown::error_renderer::{self, parse_int_in_range, Highlight, Location},
    render_context::RenderContext,
    renderable_ast::Position,
};

use super::flex::GAP_KEY;

pub type Result<T> = std::result::Result<T, Error>;

pub static COLUMNS_KEY: &str = "cols";

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.Primitives.Box")]
#[serde(rename_all = "snake_case")]
pub struct Grid {
    pub gap: usize,
    pub columns: usize,
}

impl Default for Grid {
    fn default() -> Self {
        Grid { gap: 1, columns: 2 }
    }
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid {GAP_KEY}. Expected value to be a number between 1 and 6."#)]
    InvalidGap(String),
    #[error(r#"Invalid {COLUMNS_KEY}. Expected value to be a number between 1 and 5."#)]
    InvalidColumns(String),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, node_pos: &Position) -> String {
        let mut highlights = vec![];

        match self {
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
            Error::InvalidColumns(found) => {
                let pos =
                    error_renderer::offset_attribute_error_pos(md, COLUMNS_KEY, found, node_pos);
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

impl Grid {
    pub fn try_new(gap: Option<Value>, cols: Option<Value>) -> Result<Self> {
        let mut grid = Self::default();

        if let Some(gap) = gap {
            grid.gap = parse_int_in_range(gap.to_string().as_str(), 1..6)
                .ok_or(Error::InvalidGap(gap.to_string()))?;
        }

        if let Some(cols) = cols {
            grid.columns = parse_int_in_range(cols.to_string().as_str(), 1..5)
                .ok_or(Error::InvalidColumns(cols.to_string()))?;
        }

        Ok(grid)
    }
}

impl PrimitiveComponentAutocomplete for Grid {
    fn title(&self) -> &str {
        "Grid"
    }

    fn attributes(&self) -> Vec<&str> {
        vec!["cols", "gap"]
    }

    fn attribute_values(&self, attribute: &str) -> Vec<&str> {
        match attribute {
            "cols" => vec!["1", "2", "3", "4", "5"],
            "gap" => vec!["1", "2", "3", "4", "5", "6"],
            _ => vec![],
        }
    }
}

#[cfg(test)]
mod test {
    use crate::markdown::ast_mdx;
    use crate::render_context::RenderContext;

    mod grid {
        use pretty_assertions::assert_str_eq;

        use super::*;

        #[test]
        fn columns() {
            let markdown = indoc! {r#"
            <Grid cols="3">
                <div></div>
                <div></div>
                <div></div>
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap().children[0];

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Grid gap={1} columns={3}>
                    <div>
                    </div>
                    <div>
                    </div>
                    <div>
                    </div>
                </Grid>
                "# }
            );
        }

        #[test]
        fn unexpected_attributes() {
            let markdown = indoc! {r#"
            <Grid foobar="booboo">
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Unexpected attribute");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Unexpected attribute "foobar"

                    1 │ <Grid foobar="booboo">
                              ▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn too_many_cols() {
            let markdown = indoc! {r#"
            <Grid cols="8">
                <div></div>
                <div></div>
                <div></div>
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(err.message, "Error in grid");
            assert_str_eq!(
                err.description,
                indoc! { r#"
                Invalid cols. Expected value to be a number between 1 and 5.

                    1 │ <Grid cols="8">
                                    ▲

                "# }
            )
        }

        #[test]
        fn invalid_cols() {
            let markdown = indoc! {r#"
            <Grid cols="booboo">
                <div></div>
                <div></div>
                <div></div>
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(err.message, "Error in grid");
            assert_str_eq!(
                err.description,
                indoc! { r#"
                Invalid cols. Expected value to be a number between 1 and 5.

                    1 │ <Grid cols="booboo">
                                    ▲▲▲▲▲▲

                "#},
            );
        }

        #[test]
        fn gap() {
            let markdown = indoc! {r#"
            <Grid gap="3">
                <div></div>
                <div></div>
                <div></div>
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <Grid gap={3} columns={2}>
                    <div>
                    </div>
                    <div>
                    </div>
                    <div>
                    </div>
                </Grid>
                "# }
            );
        }

        #[test]
        fn too_large_gap() {
            let markdown = indoc! {r#"
            <Grid gap="6">
                <div></div>
                <div></div>
                <div></div>
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(err.message, "Error in grid");
            assert_str_eq!(
                err.description,
                indoc! { r#"
                Invalid gap. Expected value to be a number between 1 and 6.

                    1 │ <Grid gap="6">
                                   ▲

                "# }
            )
        }

        #[test]
        fn invalid_gap() {
            let markdown = indoc! {r#"
            <Grid gap="booboo">
                <div></div>
                <div></div>
                <div></div>
            </Grid>
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(err.message, "Error in grid");
            assert_str_eq!(
                err.description,
                indoc! { r#"
                Invalid gap. Expected value to be a number between 1 and 6.

                    1 │ <Grid gap="booboo">
                                   ▲▲▲▲▲▲

                "# }
            )
        }
    }
}
