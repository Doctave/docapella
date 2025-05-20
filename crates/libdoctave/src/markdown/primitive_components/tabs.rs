use crate::{
    autocomplete::PrimitiveComponentAutocomplete,
    expressions::Value,
    markdown::error_renderer::{self, Highlight, Location},
    render_context::RenderContext,
    renderable_ast::{Node as RenderableNode, NodeKind as RenderableNodeKind, Position},
};

use thiserror::Error;

#[cfg(feature = "rustler")]
use rustler::NifStruct;

#[cfg(test)]
use ts_rs::TS;

pub static TITLE_KEY: &str = "title";

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Tabs {
    pub tabs: Vec<Tab>,
}

impl Tabs {
    pub fn verify(node: &RenderableNode) -> Result<()> {
        match &node.kind {
            RenderableNodeKind::Tab(_) => {}
            RenderableNodeKind::Text { value } => {
                if value.as_str() != "\n" && value.as_str() != " " {
                    return Err(Error::InvalidTabNode);
                }
            }
            _ => return Err(Error::InvalidTabNode),
        }

        Ok(())
    }
}

impl PrimitiveComponentAutocomplete for Tabs {
    fn title(&self) -> &str {
        "Tabs"
    }

    fn attributes(&self) -> Vec<&str> {
        vec![]
    }

    fn attribute_values(&self, _attribute: &str) -> Vec<&str> {
        vec![]
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.Markdown.Tab")]
#[serde(rename = "MdTab")]
pub struct Tab {
    pub title: String,
}

impl Tab {
    pub fn try_new(title: Option<Value>) -> Result<Self> {
        Ok(Self {
            title: title.ok_or(Error::MissingTitle)?.to_string(),
        })
    }
}

impl PrimitiveComponentAutocomplete for Tab {
    fn title(&self) -> &str {
        "Tab"
    }

    fn attributes(&self) -> Vec<&str> {
        vec!["title"]
    }

    fn attribute_values(&self, _attribute: &str) -> Vec<&str> {
        vec![]
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid tab"#)]
    InvalidTabNode,
    #[error(r#"Missing {TITLE_KEY}"#)]
    MissingTitle,
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, pos: &Position) -> String {
        let mut highlights = vec![];
        match self {
            Error::InvalidTabNode => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some("Invalid tab node".to_string()),
                };

                highlights.push(highlight);
            }
            Error::MissingTitle => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some(format!("Missing {TITLE_KEY}")),
                };

                highlights.push(highlight);
            }
        }

        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_str_eq;

    use crate::{ast_mdx, render_context::RenderContext};

    #[test]
    fn tabs_basic() {
        let input = indoc! {r#"
        <Tabs>
            <Tab title="first">Foobar 1</Tab>
            <Tab title="second">Foobar 2</Tab>
            <Tab title="third">Foobar 3</Tab>
        </Tabs>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Tabs>
                <Tab title={"first"}>
                    <Text>
                        Foobar 1
                    </Text>
                </Tab>
                <Text>


                </Text>
                <Tab title={"second"}>
                    <Text>
                        Foobar 2
                    </Text>
                </Tab>
                <Text>


                </Text>
                <Tab title={"third"}>
                    <Text>
                        Foobar 3
                    </Text>
                </Tab>
            </Tabs>
            "#}
        );
    }

    #[test]
    fn unexpected_attributes() {
        let markdown = indoc! {r#"
        <Tabs foobar="booboo">
        </Tabs>
        "#};

        let ctx = RenderContext::new();
        let error = &ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(error.message, "Unexpected attribute");
        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "foobar"

                1 │ <Tabs foobar="booboo">
                          ▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn tabs_non_tab_child() {
        let input = indoc! {r#"
        <Tabs>
            <Tab title="first">Foobar 1</Tab>
            <Box>asdf</Box>
            <Tab title="second">Foobar 2</Tab>
            <Tab title="third">Foobar 3</Tab>
        </Tabs>
        "#};

        let ctx = RenderContext::default();
        let error = &ast_mdx(input, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
              Invalid tab

                  2 │     <Tab title="first">Foobar 1</Tab>
                  3 │     <Box>asdf</Box>
                          ▲
                          └─ Invalid tab node

            "#}
        );
    }

    #[test]
    fn tabs_missing_title() {
        let input = indoc! {r#"
        <Tabs>
            <Tab title="first">Foobar 1</Tab>
            <Tab>Foobar 2</Tab>
            <Tab title="third">Foobar 3</Tab>
        </Tabs>
        "#};

        let ctx = RenderContext::default();
        let error = &ast_mdx(input, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Missing title

                2 │     <Tab title="first">Foobar 1</Tab>
                3 │     <Tab>Foobar 2</Tab>
                        ▲
                        └─ Missing title

            "#}
        );
    }

    #[test]
    fn tabs_with_conditional() {
        let input = indoc! {r#"
        <Tabs>
            <Tab if={false} title="first">Foobar 1</Tab>
            <Tab else title="second">Foobar 2</Tab>
            <Tab title="third">Foobar 3</Tab>
        </Tabs>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Tabs>
                <Tab title={"second"}>
                    <Text>
                        Foobar 2
                    </Text>
                </Tab>
                <Text>


                </Text>
                <Tab title={"third"}>
                    <Text>
                        Foobar 3
                    </Text>
                </Tab>
            </Tabs>
            "#}
        );
    }
}
