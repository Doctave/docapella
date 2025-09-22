use crate::{
    autocomplete::PrimitiveComponentAutocomplete,
    expressions::Value,
    markdown::error_renderer::{self, Highlight, Location},
    primitive_components::tabs::TITLE_KEY,
    render_context::RenderContext,
    renderable_ast::{Node as RenderableNode, NodeKind as RenderableNodeKind, Position},
};

use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Steps {
    pub steps: Vec<Step>,
}

impl Steps {
    pub fn verify(node: &RenderableNode) -> Result<()> {
        match &node.kind {
            RenderableNodeKind::Step(_) => {}
            RenderableNodeKind::Text { value } => {
                if value.as_str() != "\n" && value.as_str() != " " {
                    return Err(Error::InvalidStepNode);
                }
            }
            _ => return Err(Error::InvalidStepNode),
        }

        Ok(())
    }
}

impl PrimitiveComponentAutocomplete for Steps {
    fn title(&self) -> &str {
        "Steps"
    }

    fn attributes(&self) -> Vec<&str> {
        vec![]
    }

    fn attribute_values(&self, _attribute: &str) -> Vec<&str> {
        vec![]
    }
}

#[derive(Debug, Default, Clone, PartialEq, Serialize)]
pub struct Step {
    pub title: String,
}

impl Step {
    pub fn try_new(title: Option<Value>) -> Result<Self> {
        Ok(Self {
            title: title.ok_or(Error::MissingTitle)?.to_string(),
        })
    }
}

impl PrimitiveComponentAutocomplete for Step {
    fn title(&self) -> &str {
        "Step"
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
    #[error(r#"Invalid step"#)]
    InvalidStepNode,
    #[error(r#"Missing {TITLE_KEY}"#)]
    MissingTitle,
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, pos: &Position) -> String {
        let mut highlights = vec![];
        match self {
            Error::InvalidStepNode => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some("Invalid step node".to_string()),
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
    fn steps_basic() {
        let input = indoc! {r#"
        <Steps>
            <Step title="first">Foobar 1</Step>
            <Step title="second">Foobar 2</Step>
            <Step title="third">Foobar 3</Step>
        </Steps>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Steps>
                <Step title={"first"}>
                    <Text>
                        Foobar 1
                    </Text>
                </Step>
                <Text>


                </Text>
                <Step title={"second"}>
                    <Text>
                        Foobar 2
                    </Text>
                </Step>
                <Text>


                </Text>
                <Step title={"third"}>
                    <Text>
                        Foobar 3
                    </Text>
                </Step>
            </Steps>
            "#}
        );
    }

    #[test]
    fn unexpected_attributes() {
        let markdown = indoc! {r#"
        <Steps foobar="booboo">
        </Steps>
        "#};

        let ctx = RenderContext::new();
        let error = &ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(error.message, "Unexpected attribute");
        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "foobar"

                1 │ <Steps foobar="booboo">
                           ▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn unexpected_attributes_step() {
        let markdown = indoc! {r#"
        <Steps>
          <Step foobar="booboo">Foobar 1</Step>
        </Steps>
        "#};

        let ctx = RenderContext::new();
        let error = &ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(error.message, "Unexpected attribute");
        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "foobar"

                1 │ <Steps>
                2 │   <Step foobar="booboo">Foobar 1</Step>
                            ▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn steps_non_step_child() {
        let input = indoc! {r#"
        <Steps>
            <Step title="first">Foobar 1</Step>
            <Box>asdf</Box>
            <Step title="second">Foobar 2</Step>
            <Step title="third">Foobar 3</Step>
        </Steps>
        "#};

        let ctx = RenderContext::default();
        let error = &ast_mdx(input, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
              Invalid step

                  2 │     <Step title="first">Foobar 1</Step>
                  3 │     <Box>asdf</Box>
                          ▲
                          └─ Invalid step node

            "#}
        );
    }

    #[test]
    fn steps_missing_title() {
        let input = indoc! {r#"
        <Steps>
            <Step title="first">Foobar 1</Step>
            <Step>Foobar 2</Step>
            <Step title="third">Foobar 3</Step>
        </Steps>
        "#};

        let ctx = RenderContext::default();
        let error = &ast_mdx(input, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Missing title

                2 │     <Step title="first">Foobar 1</Step>
                3 │     <Step>Foobar 2</Step>
                        ▲
                        └─ Missing title

            "#}
        );
    }

    #[test]
    fn steps_with_conditional() {
        let input = indoc! {r#"
        <Steps>
            <Step if={false} title="first">Foobar 1</Step>
            <Step else title="second">Foobar 2</Step>
            <Step title="third">Foobar 3</Step>
        </Steps>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Steps>
                <Step title={"second"}>
                    <Text>
                        Foobar 2
                    </Text>
                </Step>
                <Text>


                </Text>
                <Step title={"third"}>
                    <Text>
                        Foobar 3
                    </Text>
                </Step>
            </Steps>
            "#}
        );
    }
}
