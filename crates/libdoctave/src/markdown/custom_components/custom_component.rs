use thiserror::Error;

use std::path::{Path, PathBuf};

use crate::{
    content_ast,
    expressions::Value,
    frontmatter,
    interpreter::Interpreter,
    markdown::{
        custom_components::attribute::{Attribute, AttributeType, AttributeTypeValue},
        error_renderer::{self, Highlight, Location},
    },
    render_context::RenderContext,
    renderable_ast::Position,
    utils::capitalize,
    Node,
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone)]
pub enum ComponentKind {
    Component,
    Topic,
    Unknown,
}

impl ComponentKind {
    pub fn from_name(name: &str) -> Self {
        if name.starts_with("Topic.") {
            Self::Topic
        } else if name.starts_with("Component.") {
            Self::Component
        } else {
            Self::Unknown
        }
    }

    pub fn from_path(path: &Path) -> Self {
        if path.starts_with("_topics/") {
            Self::Topic
        } else if path.starts_with("_components/") {
            Self::Component
        } else {
            Self::Unknown
        }
    }
}

impl std::fmt::Display for ComponentKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ComponentKind::Component => write!(f, "component"),
            ComponentKind::Topic => write!(f, "topic"),
            ComponentKind::Unknown => write!(f, "element"),
        }
    }
}

lazy_static! {
    pub(crate) static ref BAKED_COMPONENTS: Vec<CustomComponentHandle> = vec![
        CustomComponentHandle::new(
            include_str!("../composite_components/Card.md"),
            PathBuf::from("Card.md"),
        ),
        CustomComponentHandle::new(
            include_str!("../composite_components/Callout.md"),
            PathBuf::from("Callout.md")
        ),
        CustomComponentHandle::new(
            include_str!("../composite_components/Button.md"),
            PathBuf::from("Button.md")
        )
        .unwrap_lone_p(),
        CustomComponentHandle::new(
            include_str!("../composite_components/Link.md"),
            PathBuf::from("Link.md")
        )
        .unwrap_lone_p(),
        CustomComponentHandle::new(
            include_str!("../composite_components/Fragment.md"),
            PathBuf::from("Fragment.md")
        ),
        CustomComponentHandle::new(
            include_str!("../composite_components/Image.md"),
            PathBuf::from("Image.md")
        ),
        CustomComponentHandle::new(
            include_str!("../composite_components/Icon.md"),
            PathBuf::from("Icon.md")
        ),
    ];
}

#[derive(Error, Debug, Clone)]
pub(crate) enum Error {
    #[error("Invalid attribute list")]
    InvalidFrontmatter(Location, String),
    #[error("Invalid path `{0}`\n{1}")]
    InvalidComponentName(String, &'static str),
    #[error("Invalid {1}")]
    InvalidComponent(String, ComponentKind),
    #[error("Mismatch in one of: expected {1}, found {2}")]
    MismatchOneOf(Attribute, AttributeType, AttributeTypeValue),
    #[error("`<Slot />` can only be used in components and topics")]
    InvalidSlot(Position),
    #[error("Invalid title: {1}")]
    InvalidTitle(Attribute, String),
    #[error(r#"Unknown {0} "{1}""#)]
    UnknownComponent(ComponentKind, String, Position),
    #[error(r#"Stack is too deep"#)]
    RecursiveComponent(String, Position),
    #[error(r#"Unexpected attribute "{0}""#)]
    UnexpectedAttribute(String, Position),
    #[error(r#"Mismatch in default: expected {1}, found {2}"#)]
    MismatchDefault(Attribute, AttributeType, AttributeTypeValue),
    #[error(r#"Invalid default"#)]
    InvalidDefault(Attribute, String),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext) -> String {
        let (fm, _) = frontmatter::split(md);

        let highlights = match self {
            Error::InvalidFrontmatter(location, msg) => {
                let highlight = Highlight {
                    span: 1,
                    location: location.clone(),
                    msg: Some(msg.clone()),
                };

                vec![highlight]
            }
            Error::InvalidComponent(description, _kind) => {
                // We have already rendered the description in this case. Just reuse that.
                return description.clone();
            }
            Error::InvalidSlot(node_pos) => {
                let location = Location::Point(node_pos.start.row, node_pos.start.col);

                vec![Highlight {
                    location,
                    span: node_pos.end.byte_offset - node_pos.start.byte_offset,
                    msg: Some("Invalid slot".to_string()),
                }]
            }
            Error::InvalidTitle(attribute, _) => {
                let mut highlights = vec![];

                if let Some(located) = attribute.locate_from_source(fm) {
                    if let Some((_, val_loc)) = located.title_location() {
                        let highlight = Highlight {
                            location: val_loc,
                            msg: None,
                            span: attribute.title.len(),
                        };

                        highlights.push(highlight);
                    }
                }

                highlights
            }
            Error::MismatchOneOf(attribute, expected, found) => {
                let mut highlights = vec![];
                if let Some((_, val_loc)) = attribute
                    .locate_from_source(fm)
                    .and_then(|l| l.is_a_location())
                {
                    let highlight = Highlight {
                        location: val_loc,
                        msg: Some(format!("Expected {expected}")),
                        span: expected.to_string().len(),
                    };

                    highlights.push(highlight);
                }
                if let Some((_, val_loc)) = attribute
                    .locate_from_source(fm)
                    .and_then(|l| l.is_one_of_location(&found.value_as_string()))
                {
                    let highlight = Highlight {
                        location: val_loc,
                        msg: Some(format!(
                            r#"Found "{}" which is of type {found}"#,
                            found.value_as_string()
                        )),
                        span: found.value_as_string().len(),
                    };

                    highlights.push(highlight);
                }

                highlights
            }
            Error::MismatchDefault(attribute, expected, found) => {
                let mut highlights = vec![];
                if let Some((_, val_loc)) = attribute
                    .locate_from_source(fm)
                    .and_then(|l| l.is_a_location())
                {
                    let highlight = Highlight {
                        location: val_loc,
                        msg: Some(format!("Expected {expected}")),
                        span: expected.to_string().len(),
                    };

                    highlights.push(highlight);
                }
                if let Some((_, val_loc)) = attribute
                    .locate_from_source(fm)
                    .and_then(|l| l.default_location())
                {
                    let highlight = Highlight {
                        location: val_loc,
                        msg: Some(format!(
                            r#"Found "{}" which is of type {found}"#,
                            found.value_as_string()
                        )),
                        span: found.value_as_string().len(),
                    };

                    highlights.push(highlight);
                }

                highlights
            }
            Error::UnknownComponent(_kind, found, node_pos) => {
                let location = Location::Point(node_pos.start.row, node_pos.start.col + 1);

                vec![Highlight {
                    location,
                    span: found.len(),
                    msg: None,
                }]
            }
            Error::RecursiveComponent(found, node_pos) => {
                let location = Location::Point(node_pos.start.row, node_pos.start.col + 1);

                vec![Highlight {
                    location,
                    span: found.len(),
                    msg: Some("You may have a component referencing itself".to_string()),
                }]
            }
            Error::UnexpectedAttribute(found_key, node_pos) => {
                let pos = error_renderer::offset_attribute_key_error_pos(md, found_key, node_pos);

                let location = Location::Point(pos.start.row, pos.start.col);
                let span = pos.end.col - pos.start.col;

                vec![Highlight {
                    location,
                    span,
                    msg: None,
                }]
            }
            Error::InvalidDefault(attribute, found) => {
                let mut highlights = vec![];
                if let Some((_, val_loc)) = attribute
                    .locate_from_source(fm)
                    .and_then(|l| l.default_location())
                {
                    let highlight = Highlight {
                        location: val_loc,
                        msg: Some(format!("Found {found}")),
                        span: found.to_string().len(),
                    };

                    highlights.push(highlight);
                }

                if let Some(first_one_of) = attribute.validation.is_one_of.first() {
                    let first_as_string = first_one_of.value_as_string();
                    if let Some((_, val_loc)) = attribute
                        .locate_from_source(fm)
                        .and_then(|l| l.is_one_of_location(&first_as_string))
                    {
                        let highlight = Highlight {
                            location: val_loc,
                            msg: Some(format!(
                                r#"Expected one of {}"#,
                                attribute.is_one_of_string()
                            )),
                            span: first_as_string.len(),
                        };

                        highlights.push(highlight);
                    }
                }

                highlights
            }
            _ => vec![],
        };
        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }
}
#[derive(Debug, Clone)]
pub(crate) struct CustomComponentHandle {
    pub path: PathBuf,
    pub content: String,
    pub unwrap_lone_p: bool,
}

impl CustomComponentHandle {
    pub fn new<P: Into<PathBuf>>(content: &str, path: P) -> Self {
        let path = path.into();

        Self {
            path,
            content: content.to_string(),
            unwrap_lone_p: false,
        }
    }

    pub fn unwrap_lone_p(mut self) -> Self {
        self.unwrap_lone_p = true;
        self
    }

    pub fn build(&self) -> Result<CustomComponent> {
        let (frontmatter, _) = frontmatter::split(&self.content);
        let mut component: CustomComponent = serde_yaml::from_str(frontmatter).map_err(|e| {
            let location = if let Some(loc) = e.location() {
                Location::Point(loc.line(), loc.column())
            } else {
                Location::Point(0, 0)
            };
            Error::InvalidFrontmatter(location, e.to_string())
        })?;

        component.title = self.title()?;

        component.verify().map_err(|e| e[0].clone())?;

        Ok(component)
    }

    /// Returns the title of the component computed based on the path:
    ///
    ///   _components/foo.md => Component.Foo
    ///   _components/foo/bar.md => Component.Foo.Bar
    ///   _components/foo-bar/baz.md => Component.FooBar.Baz
    ///
    ///   _topics/foo.md => Topic.Foo
    ///   _topics/foo/bar.md => Topic.Foo.Bar
    ///   _topics/foo-bar/baz.md => Topic.FooBar.Baz
    ///
    /// Baked custom components are as is:
    ///   Card.md => Card
    ///
    /// NOTE: Safe to unwrap if `verify` has been called on the component
    pub fn title(&self) -> Result<String> {
        compute_title(&self.path)
    }

    pub fn matches_title(&self, title: &str) -> bool {
        self.title().map(|t| t == title).unwrap_or(false)
    }

    pub fn verify(&self, ctx: &RenderContext) -> std::result::Result<(), Vec<Error>> {
        let mut errors = vec![];
        if let Err(e) = self.title() {
            errors.push(e);
        }
        if let Err(e) = self.verify_template(ctx) {
            errors.push(e);
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    pub fn error_lines_offset(&self) -> usize {
        self.content[..self.error_bytes_offset()].lines().count()
    }

    pub fn error_bytes_offset(&self) -> usize {
        frontmatter::end_pos(&self.content)
    }

    pub fn evaluate(
        &self,
        ctx: &RenderContext,
        attr_values: Vec<(String, Value)>,
        slot_content: Vec<Node>,
        stack_depth: u8,
    ) -> crate::Result<Node> {
        let mut interpreter = Interpreter::new(ctx, frontmatter::without(&self.content));
        interpreter.slot_injection = Some(slot_content);
        interpreter.stack_depth = stack_depth + 1;

        for (key, val) in attr_values {
            interpreter.expr_interpreter.env.add_global(key, val);
        }

        let content_ast = content_ast::build_mdx(frontmatter::without(&self.content), ctx)?;

        match interpreter.interpret(content_ast) {
            Err(mut e) => {
                e.message = "Error rendering component".to_string();
                e.file = Some(self.path.clone());
                Err(e)
            }
            ok => ok,
        }
    }

    /// Verify the template syntax. This means we parse the Markdown without actually executing it,
    /// just checking that we can get a "content" ast out of it.
    ///
    /// We don't actually parse/evaluate expressions yet, because we don't have the inputs in order
    /// to that yet.
    fn verify_template(&self, ctx: &RenderContext) -> Result<()> {
        content_ast::build_mdx(frontmatter::without(&self.content), ctx)
            .map_err(|e| {
                Error::InvalidComponent(e.description, ComponentKind::from_path(&self.path))
            })
            .map(|_| ())
    }
}

#[derive(Deserialize, Debug, Clone)]
#[serde(deny_unknown_fields)]
pub(crate) struct CustomComponent {
    #[serde(default)]
    pub attributes: Vec<Attribute>,
    #[serde(skip_deserializing)]
    pub title: String,
}

impl CustomComponent {
    pub fn verify(&self) -> std::result::Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Err(e) = self.verify_validation() {
            errors.push(e);
        }
        if !errors.is_empty() {
            return Err(errors);
        }

        Ok(())
    }

    fn verify_validation(&self) -> Result<()> {
        for attribute in &self.attributes {
            attribute.verify()?;
        }

        Ok(())
    }
}

/// Computes the name for the path.
fn compute_title(original_path: &Path) -> Result<String> {
    let mut path = original_path.to_owned();
    path.set_extension("");

    let mut parts = vec![];

    for section in path.components() {
        match section.as_os_str().to_str() {
            Some("_components") => {
                parts.push("Component".to_string());
            }
            Some("_topics") => {
                parts.push("Topic".to_string());
            }
            Some(as_str) => {
                if !as_str
                    .chars()
                    .next()
                    .map(|c| c.is_ascii_alphabetic())
                    .unwrap_or(false)
                {
                    return Err(Error::InvalidComponentName(
                        original_path.display().to_string(),
                        "All parts of the component path must start with a letter",
                    ));
                }

                if !as_str
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
                {
                    return Err(Error::InvalidComponentName(
                        original_path.display().to_string(),
                        "Path can only include alphanumerics, underscores, and hyphens",
                    ));
                }

                parts.push(to_camel_case(as_str));
            }
            _ => {
                return Err(Error::InvalidComponentName(
                    original_path.display().to_string(),
                    "Path contained non-UTF8 character",
                ))
            }
        }
    }

    Ok(parts.join("."))
}

fn to_camel_case(s: &str) -> String {
    s.split(&['-', '_'][..])
        .filter(|part| !part.is_empty())
        .map(capitalize)
        .collect()
}

#[cfg(test)]
mod test {
    use crate::{ast_mdx, render_context::RenderContext, RenderOptions};

    use super::*;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn custom_component_literal_number_works() {
        let component_input = indoc! {r#"
        ---
        attributes:
          - title: pad
            required: false
            default: 3
            validation:
              is_a: number
              is_one_of:
                - 0
                - 1
                - 2
                - 3
                - 4
                - 5
        ---

        <Box pad={@pad}>
          <Slot />
        </Box>
        "#};

        let mut ctx = RenderContext::default();

        let components = [CustomComponentHandle::new(
            component_input,
            Path::new("_components/example.md"),
        )];

        ctx.custom_components = &components;

        let input = indoc! {r#"
        <Component.Example pad="5">
            Hello **world**
        </Component.Example>
        "#};

        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={5} max_width={full} class={} height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn renders_inline_components_properly() {
        let input = indoc! {r#"
        Hello <Link href="/">World</Link>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Paragraph>
                <Text>
                    Hello
                </Text>
                <a else href={/} target={_self} class={}>
                    <Text>
                        World
                    </Text>
                </a>
            </Paragraph>
            "#}
        );
    }

    #[test]
    fn renders_inline_components_properly_after() {
        let input = indoc! {r#"
        <Link href="/">World</Link> Hello
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Paragraph>
                <a else href={/} target={_self} class={}>
                    <Text>
                        World
                    </Text>
                </a>
                <Text>
                    Hello
                </Text>
            </Paragraph>
            "#}
        );
    }

    #[test]
    fn card_renders_with_default() {
        let input = indoc! {"
        <Card>
            Hello **world**
        </Card>
        "};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={3} max_width={full} class={d-card } height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn card_rewrites_href() {
        let markdown = indoc! {r#"
        <Card href="/foo">
            The text
        </Card>
        "#};

        let mut opts = RenderOptions::default();
        opts.link_rewrites
            .insert("/foo".to_string(), "/bar".to_string());

        let mut ctx = RenderContext::new();
        ctx.options = &opts;

        let node = &ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! { r#"
            <a class={d-card-link} if={/foo} href={/bar}>
                <Box padding={3} max_width={full} class={d-card d-hover } height={Auto}>
                    <Paragraph>
                        <Text>
                            The text
                        </Text>
                    </Paragraph>
                </Box>
            </a>
            "# }
        );
    }

    #[test]
    fn card_keeps_fragment_link() {
        let markdown = indoc! {r##"
        <Card href="./foo#foobar">
            The text
        </Card>
        "##};

        let mut opts = RenderOptions::default();
        opts.link_rewrites
            .insert("/docs/foo".to_string(), "/docs/bar".to_string());

        let mut ctx = RenderContext::new();
        ctx.options = &opts;
        ctx.relative_url_base = Some("/docs".to_string());

        let node = &ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! { r#"
            <a class={d-card-link} if={./foo#foobar} href={/docs/bar#foobar}>
                <Box padding={3} max_width={full} class={d-card d-hover } height={Auto}>
                    <Paragraph>
                        <Text>
                            The text
                        </Text>
                    </Paragraph>
                </Box>
            </a>
            "# }
        );
    }

    #[test]
    fn card_webbifies_href() {
        let markdown = indoc! {r#"
        <Card href="/foo.md">
            The text
        </Card>
        "#};

        let opts = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.options = &opts;

        let node = &ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! { r#"
            <a class={d-card-link} if={/foo.md} href={/foo}>
                <Box padding={3} max_width={full} class={d-card d-hover } height={Auto}>
                    <Paragraph>
                        <Text>
                            The text
                        </Text>
                    </Paragraph>
                </Box>
            </a>
            "# }
        );
    }

    #[test]
    fn card_prefixes_href() {
        let markdown = indoc! {r#"
        <Card href="/foo">
            The text
        </Card>
        "#};

        let opts = RenderOptions {
            prefix_link_urls: Some("/docs".to_string()),
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.options = &opts;

        let node = &ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! { r#"
            <a class={d-card-link} if={/foo} href={/docs/foo}>
                <Box padding={3} max_width={full} class={d-card d-hover } height={Auto}>
                    <Paragraph>
                        <Text>
                            The text
                        </Text>
                    </Paragraph>
                </Box>
            </a>
            "# }
        );
    }

    #[test]
    fn card_renders_with_padding() {
        let input = indoc! {r#"
        <Card pad={5}>
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={5} max_width={full} class={d-card } height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn card_errors_with_padding() {
        let input = indoc! {r#"
        <Card pad="100">
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let error = &ast_mdx(input, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected value. Found `100`, expected one of 0, 1, 2, 3, 4, 5

                1 │ <Card pad="100">
                               ▲▲▲

            "#}
        );
    }

    #[test]
    fn card_renders_with_max_width() {
        let input = indoc! {r#"
        <Card max_width="md">
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={3} max_width={md} class={d-card } height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn card_errors_with_max_width() {
        let input = indoc! {r#"
        <Card max_width="foo">
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let error = &ast_mdx(input, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected value. Found `foo`, expected one of sm, md, lg, xl, full

                1 │ <Card max_width="foo">
                                     ▲▲▲

            "#}
        );
    }

    #[test]
    fn card_renders_with_href() {
        let input = indoc! {r#"
        <Card href="/foobar">
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <a class={d-card-link} if={/foobar} href={/foobar}>
                <Box padding={3} max_width={full} class={d-card d-hover } height={Auto}>
                    <Paragraph>
                        <Text>
                            Hello
                        </Text>
                        <Strong>
                            <Text>
                                world
                            </Text>
                        </Strong>
                    </Paragraph>
                </Box>
            </a>
            "#}
        );
    }

    #[test]
    fn callout_renders_with_default() {
        let input = indoc! {"
        <Callout>
            Hello **world**
        </Callout>
        "};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={2} max_width={full} class={d-callout d-callout-info} height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn callout_renders_with_type() {
        let input = indoc! {r#"
        <Callout type="warning">
            Hello **world**
        </Callout>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={2} max_width={full} class={d-callout d-callout-warning} height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn callout_renders_with_padding() {
        let input = indoc! {r#"
        <Callout pad="4">
            Hello **world**
        </Callout>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={4} max_width={full} class={d-callout d-callout-info} height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn card_renders_with_custom_class() {
        let input = indoc! {r#"
        <Card class="custom">
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={3} max_width={full} class={d-card custom} height={Auto}>
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                    <Strong>
                        <Text>
                            world
                        </Text>
                    </Strong>
                </Paragraph>
            </Box>
            "#}
        );
    }

    #[test]
    fn card_renders_with_custom_class_with_href() {
        let input = indoc! {r#"
        <Card href="/foobar" class="custom">
            Hello **world**
        </Card>
        "#};

        let ctx = RenderContext::default();
        let node = &ast_mdx(input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <a class={d-card-link} if={/foobar} href={/foobar}>
                <Box padding={3} max_width={full} class={d-card d-hover custom} height={Auto}>
                    <Paragraph>
                        <Text>
                            Hello
                        </Text>
                        <Strong>
                            <Text>
                                world
                            </Text>
                        </Strong>
                    </Paragraph>
                </Box>
            </a>
            "#}
        );
    }

    #[test]
    fn parses_is_one_of() {
        let input = indoc! {r#"
        ---
        attributes:
          - title: heading
            required: true
            validation:
              is_a: boolean
              is_one_of:
                - true
        ---

        <Box></Box>
        "#};

        let component = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap();

        assert_eq!(
            component.attributes[0].validation.is_a,
            AttributeType::Boolean
        );
        assert_eq!(
            component.attributes[0].validation.is_one_of[0],
            AttributeTypeValue::Boolean(true)
        );
    }

    #[test]
    fn validates_is_one_of_error() {
        let input = indoc! {r#"
        ---
        attributes:
          - title: heading
            required: true
            validation:
              is_a: boolean
              is_one_of:
                - foobar
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();

        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! { r#"
            Mismatch in one of: expected boolean, found text

                5 │     validation:
                6 │       is_a: boolean
                                ▲▲▲▲▲▲▲
                                └─ Expected boolean
                7 │       is_one_of:
                8 │         - foobar
                              ▲▲▲▲▲▲
                              └─ Found "foobar" which is of type text

            "# }
        );
    }

    #[test]
    fn validates_is_one_of_default_error() {
        let input = indoc! {r#"
        ---
        attributes:
          - title: heading
            required: true
            default: asdf
            validation:
              is_a: text
              is_one_of:
                - foobar
                - baz
                - bar
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();

        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! { r#"
            Invalid default

                4 │     required: true
                5 │     default: asdf
                                 ▲▲▲▲
                                 └─ Found asdf
                8 │       is_one_of:
                9 │         - foobar
                              ▲▲▲▲▲▲
                              └─ Expected one of foobar, baz, bar

            "# }
        );
    }

    #[test]
    fn validates_is_a_default_error() {
        let input = indoc! {r#"
        ---
        attributes:
          - title: heading
            required: true
            default: asdf
            validation:
              is_a: boolean
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();

        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! { r#"
            Mismatch in default: expected boolean, found text

                4 │     required: true
                5 │     default: asdf
                                 ▲▲▲▲
                                 └─ Found "asdf" which is of type text
                6 │     validation:
                7 │       is_a: boolean
                                ▲▲▲▲▲▲▲
                                └─ Expected boolean

            "# }
        );
    }

    #[test]
    fn parses_attributes() {
        let input = indoc! {r#"
        ---
        attributes:
          - title: heading
            required: true
          - title: description
        ---

        <Box></Box>
        "#};

        let component = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap();

        assert_eq!(&component.attributes[0].title, "heading");
        assert!(component.attributes[0].required);
        assert_eq!(&component.attributes[1].title, "description");
        assert!(!component.attributes[1].required);
        assert!(component.verify().is_ok());
    }

    #[test]
    fn parses_the_path_to_a_title() {
        let input = indoc! {r#"
        ---
        attributes:
          - title: heading
            required: true
          - title: description
        ---
        <Box></Box>
        "#};

        let component = CustomComponentHandle::new(input, Path::new("_components/example.md"));
        assert_eq!(&component.title().unwrap(), "Component.Example");

        let component =
            CustomComponentHandle::new(input, Path::new("_components/another/example.md"));
        assert_eq!(&component.title().unwrap(), "Component.Another.Example");
        assert!(component.verify(&RenderContext::new()).is_ok());
    }

    #[test]
    fn camel_cases_the_path_sections_in_the_title() {
        let input = indoc! {r#"
        <Box></Box>
        "#};

        let component = CustomComponentHandle::new(input, Path::new("_components/example_one.md"));
        assert_eq!(&component.title().unwrap(), "Component.ExampleOne");
        assert!(component.verify(&RenderContext::new()).is_ok());

        let component =
            CustomComponentHandle::new(input, Path::new("_components/another-one/example-two.md"));
        assert_eq!(
            &component.title().unwrap(),
            "Component.AnotherOne.ExampleTwo"
        );
        assert!(component.verify(&RenderContext::new()).is_ok());
    }

    #[test]
    fn rejects_paths_sections_that_start_with_numbers() {
        let input = indoc! {r#"
        <Box></Box>
        "#};

        let error = CustomComponentHandle::new(input, Path::new("_components/2example.md"))
            .build()
            .unwrap_err();

        assert_str_eq!(error.to_string(), "Invalid path `_components/2example.md`\nAll parts of the component path must start with a letter");
    }

    #[test]
    fn rejects_paths_sections_with_non_alphanumerics() {
        let input = indoc! {r#"
        <Box></Box>
        "#};

        let error = CustomComponentHandle::new(input, Path::new("_components/example##.md"))
            .build()
            .unwrap_err();
        assert_str_eq!(error.to_string(), "Invalid path `_components/example##.md`\nPath can only include alphanumerics, underscores, and hyphens");
    }

    #[test]
    #[cfg(unix)]
    fn rejects_non_utf8_paths() {
        use std::ffi::{OsStr, OsString};
        use std::os::unix::ffi::OsStrExt;
        use std::os::unix::ffi::OsStringExt;

        let input = indoc! {r#"
        <Box></Box>
        "#};

        // Path prefix as a &str, which is valid UTF-8
        let path_prefix = "_components/";

        // Non-UTF-8 filename as a byte sequence, e.g., "example�.md"
        // Here, 0x80 represents a non-UTF-8 byte
        let non_utf8_filename = vec![
            0x65, 0x78, 0x61, 0x6d, 0x70, 0x6c, 0x65, // "example"
            0x80, // Non-UTF-8 byte
            0x2e, 0x6d, 0x64, // ".md"
        ];

        // Convert the non-UTF-8 filename to an OsString
        let os_string_filename = OsString::from_vec(non_utf8_filename);

        // Create a PathBuf from the prefix
        let mut path = PathBuf::from(path_prefix);

        // Append the non-UTF-8 filename to the path
        path.push(OsStr::from_bytes(&os_string_filename.into_vec()));

        let error = CustomComponentHandle::new(input, &path)
            .build()
            .unwrap_err();

        assert_str_eq!(
            error.to_string(),
            indoc! { r#"
                Invalid path `_components/example�.md`
                Path contained non-UTF8 character"#}
        );
    }

    #[test]
    fn error_if_invalid_frotmatter() {
        let input = indoc! {r#"
        ---
        attributes: 123
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();

        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! {r#"
            Invalid attribute list

                1 │ ---
                2 │ attributes: 123
                                ▲
                                └─ attributes: invalid type: integer `123`, expected a sequence at line 2 column 13

            "#}
        );
    }

    #[test]
    fn missing_title() {
        let input = indoc! {r#"
        ---
        attributes:
           - tasdf: asd
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();
        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! {r#"
            Invalid attribute list

                2 │ attributes:
                3 │    - tasdf: asd
                         ▲
                         └─ attributes[0]: missing field `title` at line 3 column 6

            "#}
        );
    }

    #[test]
    fn invalid_title() {
        let input = indoc! {r#"
        ---
        attributes:
           - title: 123
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();
        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! {r#"
            Invalid title: Title can only include alphabets and underscores

                2 │ attributes:
                3 │    - title: 123
                                ▲▲▲

            "#}
        );
    }

    #[test]
    fn invalid_title_format() {
        let input = indoc! {r#"
        ---
        attributes:
           - title: 99ff###
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();
        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! {r#"
            Invalid title: Title can only include alphabets and underscores

                2 │ attributes:
                3 │    - title: 99ff###
                                ▲▲▲▲▲▲▲

        "#}
        );
    }

    #[test]
    fn validates_the_component_template_during_verification() {
        let input = indoc! {r#"
        ---
        ---

        <Box></Flex>
        "#};
        let ctx = &RenderContext::default();
        let handle = CustomComponentHandle::new(input, Path::new("_components/example.md"));
        let error = &handle.verify(ctx).unwrap_err()[0];

        assert_str_eq!(
            error.render(input, ctx),
            indoc! {r#"
            Unexpected closing tag `</Flex>`, expected corresponding closing tag for `<Box>`

                1 │
                2 │ <Box></Flex>
                    ▲    ▲
                    │    ╵
                    └─ Opening tag
                         ╷
                         └─ Expected close tag

            "#},
        );
    }

    #[test]
    fn errors_on_unknown_keys() {
        let input = indoc! {r#"
        ---
        foo: bar
        attributes:
           - title: foo
        ---

        <Box></Box>
        "#};

        let err = CustomComponentHandle::new(input, Path::new("_components/example.md"))
            .build()
            .unwrap_err();
        assert_str_eq!(
            err.render(input, &RenderContext::default()),
            indoc! {r#"
            Invalid attribute list

                1 │ ---
                2 │ foo: bar
                    ▲
                    └─ unknown field `foo`, expected `attributes` at line 2 column 1

            "#}
        );
    }

    #[test]
    fn attributes_default() {
        let component_input = indoc! {r#"
        ---
        attributes:
           - title: max_width
             required: false
             default: md
        ---

        <Box max_width={@max_width}></Box>
        "#};

        let mut ctx = RenderContext::default();

        let components = [CustomComponentHandle::new(
            component_input,
            Path::new("_components/example.md"),
        )];

        ctx.custom_components = &components;

        let component_input = indoc! {r#"
        <Component.Example></Component.Example>
        "#};

        let node = &ast_mdx(component_input, &ctx).unwrap();

        assert_str_eq!(
            node.debug_string().unwrap(),
            indoc! {r#"
            <Box padding={0} max_width={md} class={} height={Auto}>
            </Box>
            "#}
        );
    }

    mod button {
        use super::*;

        #[test]
        fn href() {
            let markdown = indoc! {r#"
            <Button href="/foo">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/foo} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn target() {
            let markdown = indoc! {r#"
            <Button href="/foo" target="_blank">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_blank} href={/foo} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn download() {
            let markdown = indoc! {r#"
            <Button href="/foo" download>
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a elseif={true} download={} class={d-button} href={/foo} data-variant={primary} data-size={md} data-width={fit-content} target={_blank}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn download_as() {
            let markdown = indoc! {r#"
            <Button href="/foo" download_as="foo">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a if={foo} download={foo} class={d-button} href={/foo} data-variant={primary} data-size={md} data-width={fit-content} target={_blank}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn rewrites_button_href() {
            let markdown = indoc! {r#"
            <Button href="/foo">
                The text
            </Button>
            "#};

            let mut opts = RenderOptions::default();
            opts.link_rewrites
                .insert("/foo".to_string(), "/bar".to_string());

            let mut ctx = RenderContext::new();
            ctx.options = &opts;

            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/bar} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn webbifies_button_href() {
            let markdown = indoc! {r#"
            <Button href="/foo.md">
                The text
            </Button>
            "#};

            let opts = RenderOptions {
                webbify_internal_urls: true,
                ..RenderOptions::default()
            };

            let mut ctx = RenderContext::new();
            ctx.options = &opts;

            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/foo} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn prefixes_button_href() {
            let markdown = indoc! {r#"
            <Button href="/foo">
                The text
            </Button>
            "#};

            let opts = RenderOptions {
                prefix_link_urls: Some("/docs".to_string()),
                ..RenderOptions::default()
            };

            let mut ctx = RenderContext::new();
            ctx.options = &opts;

            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/docs/foo} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn button_keeps_fragment_link() {
            let markdown = indoc! {r##"
            <Button href="./foo#foobar">
                The text
            </Button>
            "##};

            let mut opts = RenderOptions::default();
            opts.link_rewrites
                .insert("/docs/foo".to_string(), "/docs/bar".to_string());

            let mut ctx = RenderContext::new();
            ctx.options = &opts;
            ctx.relative_url_base = Some("/docs".to_string());

            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/docs/bar#foobar} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn missing_href() {
            let markdown = indoc! {r#"
            <Button>
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! { r#"
                Missing required attribute `href` for component `Button`

                    1 │ <Button>
                        ▲
                        └─ Missing required attribute

                "# }
            );
        }

        #[test]
        fn variant_defaults_to_primary() {
            let markdown = indoc! {r#"
            <Button href="/">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn variant_can_be_overridden() {
            let markdown = indoc! {r#"
            <Button href="/" variant="secondary">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/} data-variant={secondary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn variant_is_validated() {
            let markdown = indoc! {r#"
            <Button href="/" variant="lolnope">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! { r#"
                Unexpected value. Found `lolnope`, expected one of primary, secondary, ghost, outline

                    1 │ <Button href="/" variant="lolnope">
                                                  ▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn size_defaults_to_primary() {
            let markdown = indoc! {r#"
            <Button href="/">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn size_can_be_overridden() {
            let markdown = indoc! {r#"
            <Button href="/" size="lg">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/} data-variant={primary} data-size={lg} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn size_is_validated() {
            let markdown = indoc! {r#"
            <Button href="/" size="lolnope">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! { r#"
                Unexpected value. Found `lolnope`, expected one of sm, md, lg

                    1 │ <Button href="/" size="lolnope">
                                               ▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn remove_lonely_p_tag() {
            let markdown = indoc! {r#"
            <Button href="/foo">
                The text
            </Button>
            "#};

            let ctx = RenderContext::new();
            let node = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/foo} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }

        #[test]
        fn remove_lonely_p_tag_from_inline() {
            let markdown = indoc! {r#"
            <Button href="/foo">The text</Button>
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <a else class={d-button} target={_self} href={/foo} data-variant={primary} data-size={md} data-width={fit-content}>
                    <Text>
                        The text
                    </Text>
                </a>
                "# }
            );
        }
    }
    mod image {
        use std::collections::HashMap;

        use super::*;

        #[test]
        fn can_create_a_dark_and_light_mode_image() {
            let markdown = indoc! {r#"
            <Image src="/_assets/light.png" src_dark="/_assets/dark.png" alt="Example" />
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <img if={/_assets/light.png} src={/_assets/light.png} alt={Example} class={light-only } data-zoomable={true}>
                </img>
                <img if={/_assets/dark.png} src={/_assets/dark.png} alt={Example} class={dark-only } data-zoomable={true}>
                </img>
                "#}
            );
        }

        #[test]
        fn shows_light_mode_always_if_dark_mode_src_not_given() {
            let markdown = indoc! {r#"
            <Image src="/_assets/light.png" alt="Example" />
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <img elseif={/_assets/light.png} src={/_assets/light.png} alt={Example} class={} data-zoomable={true}>
                </img>
                "#}
            );
        }

        #[test]
        fn can_add_custom_class() {
            let markdown = indoc! {r#"
            <Image src="/_assets/light.png" alt="Example" class="foo" />
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <img elseif={/_assets/light.png} src={/_assets/light.png} alt={Example} class={foo} data-zoomable={true}>
                </img>
                "#}
            );
        }

        #[test]
        fn can_disable_zoom() {
            let markdown = indoc! {r#"
            <Image src="/_assets/light.png" alt="Example" zoomable={false} />
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <img elseif={/_assets/light.png} src={/_assets/light.png} alt={Example} class={} data-zoomable={false}>
                </img>
                "#}
            );
        }

        #[test]
        fn can_add_custom_class_dark() {
            let markdown = indoc! {r#"
            <Image src="/_assets/light.png" src_dark="/_assets/dark.png" alt="Example" class="foo" />
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <img if={/_assets/light.png} src={/_assets/light.png} alt={Example} class={light-only foo} data-zoomable={true}>
                </img>
                <img if={/_assets/dark.png} src={/_assets/dark.png} alt={Example} class={dark-only foo} data-zoomable={true}>
                </img>
                "#}
            );
        }

        #[test]
        fn must_have_src() {
            let markdown = indoc! {r#"
            <Image alt="Example" />
            "#};

            let ctx = RenderContext::new();
            let error = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                error.description,
                indoc! {r#"
                Missing required attribute `src` for component `Image`

                    1 │ <Image alt="Example" />
                        ▲
                        └─ Missing required attribute

                "#}
            );
        }

        #[test]
        fn rewrites_links() {
            let markdown = indoc! {r#"
            <Image src="/_assets/light.png" src_dark="/_assets/dark.png" />
            "#};

            let ctx = RenderContext {
                options: &RenderOptions {
                    link_rewrites: HashMap::from([
                        (
                            "/_assets/light.png".to_string(),
                            "/_assets/light-prod.png".to_string(),
                        ),
                        (
                            "/_assets/dark.png".to_string(),
                            "/_assets/dark-prod.png".to_string(),
                        ),
                    ]),
                    ..Default::default()
                },
                ..Default::default()
            };
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <img if={/_assets/light.png} src={/_assets/light-prod.png} alt={} class={light-only } data-zoomable={true}>
                </img>
                <img if={/_assets/dark.png} src={/_assets/dark-prod.png} alt={} class={dark-only } data-zoomable={true}>
                </img>
                "#}
            );
        }
    }

    mod icon {
        use super::*;

        #[test]
        fn devicon() {
            let markdown = indoc! {r#"
            <Icon set="devicon" name="ruby" />
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <div if={true} role={img} aria-label={ruby icon} data-d-component={Icon} data-color={false} data-size={md} class={d-icon } style={mask-image: url(https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/ruby/ruby-plain.svg); -webkit-mask-image: url(https://cdn.jsdelivr.net/gh/devicons/devicon@latest/icons/ruby/ruby-plain.svg);}>
                </div>
                "# }
            );
        }

        #[test]
        fn lucide() {
            let markdown = indoc! {r#"
            <Icon set="lucide" name="book" />
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <div if={true} role={img} aria-label={book icon} data-d-component={Icon} data-color={false} data-size={md} class={d-icon } style={mask-image: url(https://unpkg.com/lucide-static@latest/icons/book.svg); -webkit-mask-image: url(https://unpkg.com/lucide-static@latest/icons/book.svg);}>
                </div>
                "# }
            );
        }

        #[test]
        fn set_is_required() {
            let markdown = indoc! {r#"
            <Icon name="book" />
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                err.description,
                indoc! { r#"
                Missing required attribute `set` for component `Icon`

                    1 │ <Icon name="book" />
                        ▲
                        └─ Missing required attribute

                "# }
            );
        }

        #[test]
        fn variant_must_be_valid() {
            let markdown = indoc! {r#"
            <Icon set="lucide" name="book" variant="lol" />
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                err.description,
                indoc! { r#"
                Unexpected value. Found `lol`, expected one of plain, boxed

                    1 │ <Icon set="lucide" name="book" variant="lol" />
                                                                ▲▲▲

                "# }
            );
        }

        #[test]
        fn size_must_be_valid() {
            let markdown = indoc! {r#"
            <Icon set="lucide" name="book" size="lol" />
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                err.description,
                indoc! { r#"
                Unexpected value. Found `lol`, expected one of sm, md, lg, xl

                    1 │ <Icon set="lucide" name="book" size="lol" />
                                                             ▲▲▲

                "# }
            );
        }

        #[test]
        fn set_must_be_valid() {
            let markdown = indoc! {r#"
            <Icon set="nope" name="book" />
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                err.description,
                indoc! { r#"
                Unexpected value. Found `nope`, expected one of devicon, lucide

                    1 │ <Icon set="nope" name="book" />
                                   ▲▲▲▲

                "# }
            );
        }

        #[test]
        fn name_is_required() {
            let markdown = indoc! {r#"
            <Icon set="lucide" />
            "#};

            let ctx = RenderContext::new();
            let err = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(
                err.description,
                indoc! { r#"
                Missing required attribute `name` for component `Icon`

                    1 │ <Icon set="lucide" />
                        ▲
                        └─ Missing required attribute

                "# }
            );
        }

        #[test]
        fn supports_custom_class() {
            let markdown = indoc! {r#"
            <Icon set="lucide" name="book" class="some-class" />
            "#};

            let ctx = RenderContext::new();
            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! { r#"
                <div if={true} role={img} aria-label={book icon} data-d-component={Icon} data-color={false} data-size={md} class={d-icon some-class} style={mask-image: url(https://unpkg.com/lucide-static@latest/icons/book.svg); -webkit-mask-image: url(https://unpkg.com/lucide-static@latest/icons/book.svg);}>
                </div>
                "# }
            );
        }
    }

    mod fragment {
        use super::*;

        #[test]
        fn wraps_the_content_in_a_new_root_node() {
            let markdown = indoc! {"
            <Fragment>
                The text
            </Fragment>
            "};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {"
                <Paragraph>
                    <Text>
                        The text
                    </Text>
                </Paragraph>
                "}
            );
        }

        #[test]
        fn wraps_the_content_in_a_new_root_node_inline() {
            let markdown = indoc! {"
            <Fragment>The text</Fragment>
            "};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {"
                <Text>
                    The text
                </Text>
                "}
            );
        }

        #[test]
        fn conditionally_renders_the_children() {
            let markdown = indoc! {"
            <Fragment if={1 > 2}>The text</Fragment>
            "};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(root.debug_string().unwrap(), indoc! {""});
        }
    }

    #[test]
    fn unknown_element() {
        let markdown = indoc! {"
        <Asdf>The text</Asdf>
        "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unknown element "Asdf"

                1 │ <Asdf>The text</Asdf>
                     ▲▲▲▲

            "#}
        );
    }

    #[test]
    fn unknown_component() {
        let markdown = indoc! {"
        <Component.Asdf>The text</Component.Asdf>
        "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unknown component "Component.Asdf"

                1 │ <Component.Asdf>The text</Component.Asdf>
                     ▲▲▲▲▲▲▲▲▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn unknown_topic() {
        let markdown = indoc! {"
        <Topic.Foo>The text</Topic.Foo>
        "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unknown topic "Topic.Foo"

                1 │ <Topic.Foo>The text</Topic.Foo>
                     ▲▲▲▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn unexpected_attributes() {
        let markdown = indoc! {r#"
        <Button href="/foo" asdf="123">The text</Button>
        "#};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "asdf"

                1 │ <Button href="/foo" asdf="123">The text</Button>
                                        ▲▲▲▲

            "# }
        );
    }

    #[test]
    fn unexpected_attributes_without_value() {
        let markdown = indoc! {r#"
        <Button href="/foo" asdf>The text</Button>
        "#};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "asdf"

                1 │ <Button href="/foo" asdf>The text</Button>
                                        ▲▲▲▲

            "# }
        );
    }

    #[test]
    fn unexpected_attributes_with_single_quote() {
        let markdown = indoc! {r#"
        <Button href="/foo" asdf='foobar'>The text</Button>
        "#};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "asdf"

                1 │ <Button href="/foo" asdf='foobar'>The text</Button>
                                        ▲▲▲▲

            "# }
        );
    }

    #[test]
    fn unexpected_attributes_with_curly_brace() {
        let markdown = indoc! {r#"
        <Button href="/foo" asdf={"foobar"}>The text</Button>
        "#};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Unexpected attribute "asdf"

                1 │ <Button href="/foo" asdf={"foobar"}>The text</Button>
                                        ▲▲▲▲

            "# }
        );
    }

    #[test]
    fn recursive_component() {
        let component_input = indoc! {r#"
        ---
        attributes:
          - title: max_width
            required: false
            default: md
        ---

        <Component.Foo max_width={@max_width}></Component.Foo>
        "#};

        let component2_input = indoc! {r#"
        ---
        attributes:
          - title: max_width
            required: false
            default: md
        ---
        <Component.Bar max_width={@max_width}></Component.Bar>
        "#};

        let component3_input = indoc! {r#"
        ---
        attributes:
          - title: max_width
            required: false
            default: md
        ---
        <Card>
          <Component.Example max_width={@max_width}></Component.Example>
        </Card>
        "#};

        let mut ctx = RenderContext::default();

        let components = [
            CustomComponentHandle::new(component_input, Path::new("_components/example.md")),
            CustomComponentHandle::new(component2_input, Path::new("_components/foo.md")),
            CustomComponentHandle::new(component3_input, Path::new("_components/bar.md")),
            CustomComponentHandle::new(
                include_str!("../composite_components/Card.md"),
                PathBuf::from("Card.md"),
            ),
        ];

        ctx.custom_components = &components;

        let component_input = indoc! {r#"
        <Component.Example></Component.Example>
        "#};

        let mb_1 = 1024 * 1024;
        stacker::grow(mb_1 * 15, || {
            let error = &ast_mdx(component_input, &ctx).unwrap_err();
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Stack is too deep

                    7 │ <Card>
                         ▲▲▲▲
                         └─ You may have a component referencing itself

                "#}
            );
        });
    }

    #[test]
    fn bug_incorrect_recursion() {
        let markdown = indoc! {r#"
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
          <Card></Card>
        "#};

        let ctx = RenderContext::new();

        assert!(&ast_mdx(markdown, &ctx).is_ok());
    }

    mod link {
        use super::*;

        #[test]
        fn renders_a_link() {
            let markdown = indoc! {r#"
            <Link href="https://example.com">The text</Link>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <a else href={https://example.com} target={_self} class={}>
                    <Text>
                        The text
                    </Text>
                </a>
                "#}
            );
        }

        #[test]
        fn renders_a_link_with_download() {
            let markdown = indoc! {r#"
            <Link download href="https://example.com">The text</Link>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <a elseif={true} download={} href={https://example.com} class={}>
                    <Text>
                        The text
                    </Text>
                </a>
                "#}
            );
        }

        #[test]
        fn renders_a_link_with_download_as() {
            let markdown = indoc! {r#"
            <Link download_as="foo" href="https://example.com">The text</Link>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <a if={foo} download={foo} href={https://example.com} class={}>
                    <Text>
                        The text
                    </Text>
                </a>
                "#}
            );
        }

        #[test]
        fn renders_a_link_with_target() {
            let markdown = indoc! {r#"
            <Link href="https://example.com" target="_blank">The text</Link>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <a else href={https://example.com} target={_blank} class={}>
                    <Text>
                        The text
                    </Text>
                </a>
                "#}
            );
        }

        #[test]
        fn renders_a_link_with_class() {
            let markdown = indoc! {r#"
            <Link href="https://example.com" class="my-class">The text</Link>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! {r#"
                <a else href={https://example.com} target={_self} class={my-class}>
                    <Text>
                        The text
                    </Text>
                </a>
                "#}
            );
        }

        #[test]
        fn link_rewrites_href() {
            let markdown = indoc! {r#"
            <Link href="/foo">The text</Link>
            "#};

            let mut opts = RenderOptions::default();
            opts.link_rewrites
                .insert("/foo".to_string(), "/bar".to_string());

            let mut ctx = RenderContext::new();
            ctx.options = &opts;

            let node = &ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <a else href={/bar} target={_self} class={}>
                    <Text>
                        The text
                    </Text>
                </a>
                "#}
            );
        }
    }
}
