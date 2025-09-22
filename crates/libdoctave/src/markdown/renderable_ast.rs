/// Nodes for the final renderable AST. All nodes here will be convertible to HTML
/// and there are no "virtual" nodes or expressions to evaluate.
use serde::Serialize;
use std::io::Write;

use crate::{
    open_api::ast::SchemaAst,
    primitive_components::{CBox, Flex, Grid, Step, Tab},
};

pub use super::shared_ast::*;

#[derive(Debug, Default, Clone, Serialize)]
pub struct Node {
    pub kind: NodeKind,
    #[serde(skip_serializing)]
    pub pos: Position,
    pub children: Vec<Node>,
}

impl Node {
    pub fn debug_string(&self) -> std::io::Result<String> {
        let mut f = Vec::new();

        self._debug_string(0, &mut f)?;

        Ok(String::from_utf8(f).unwrap())
    }

    fn _debug_string(&self, indent: usize, f: &mut Vec<u8>) -> std::io::Result<()> {
        let tab_size = 4;
        let i = " ".repeat(indent * tab_size);
        let next_i = " ".repeat((indent + 1) * tab_size);

        match &self.kind {
            NodeKind::Root => {
                for child in &self.children {
                    child._debug_string(indent, f)?;
                }
            }
            NodeKind::Math {
                value,
                display_mode,
            } => {
                write!(f, "{i}<Math")?;
                write!(f, " display_mode={{{display_mode}}}")?;
                writeln!(f, ">")?;

                for part in value.split('\n') {
                    let part = part.trim();
                    if part.is_empty() {
                        writeln!(f, "{part}")?;
                    } else {
                        writeln!(f, "{next_i}{}", part.trim_end())?;
                    }
                }

                writeln!(f, "{i}</Math>")?;
            }
            NodeKind::InlineMath { value } => {
                writeln!(f, "{i}<InlineMath>")?;

                for part in value.split('\n') {
                    let part = part.trim();
                    if part.is_empty() {
                        writeln!(f, "{part}")?;
                    } else {
                        writeln!(f, "{next_i}{}", part.trim_end())?;
                    }
                }

                writeln!(f, "{i}</InlineMath>")?;
            }
            NodeKind::Paragraph => {
                writeln!(f, "{i}<Paragraph>")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Paragraph>")?;
            }
            NodeKind::Tabs => {
                writeln!(f, "{i}<Tabs>")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Tabs>")?;
            }
            NodeKind::Tab(tab) => {
                write!(f, "{i}<Tab")?;
                write!(f, " title={{{:?}}}", &tab.title)?;

                writeln!(f, ">")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Tab>")?
            }
            NodeKind::Steps => {
                writeln!(f, "{i}<Steps>")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Steps>")?;
            }
            NodeKind::Step(step) => {
                write!(f, "{i}<Step")?;
                write!(f, " title={{{:?}}}", &step.title)?;
                writeln!(f, ">")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Step>")?
            }
            NodeKind::Table { .. } => {
                write!(f, "{i}<Table> ")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Table>")?;
            }
            NodeKind::Flex(flex) => {
                write!(f, "{i}<Flex")?;

                write!(f, " justify={{{:?}}}", flex.justify)?;
                write!(f, " align={{{:?}}}", flex.align)?;
                write!(f, " direction={{{:?}}}", flex.direction)?;
                write!(f, " wrap={{{:?}}}", flex.wrap)?;
                write!(f, " gap={{{}}}", flex.gap)?;
                write!(f, " padding={{{}}}", flex.padding)?;
                write!(f, " height={{{:?}}}", flex.height)?;
                write!(f, " class={{{}}}", flex.class)?;

                writeln!(f, ">")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</Flex>")?;
            }
            NodeKind::Image { url, alt, title } => {
                write!(f, "{i}<Image url={{{url}}} alt={{{alt}}}")?;

                if let Some(title) = title {
                    write!(f, " title={{{title}}}")?;
                }

                writeln!(f, " />")?;
            }
            NodeKind::Code {
                value,
                language,
                title,
                label,
                raw,
                show_whitespace,
                rendered_value: _,
            } => {
                write!(f, "{i}<Code")?;

                if let Some(language) = language {
                    write!(f, " language={{{language}}}")?;
                }

                if let Some(title) = title {
                    write!(f, " title={{{title}}}")?;
                }

                if let Some(label) = label {
                    write!(f, " label={{{label}}}")?;
                }

                write!(f, " raw={{{raw}}}")?;
                write!(f, " show_whitespace={{{show_whitespace}}}")?;

                writeln!(f, ">")?;
                for part in value.split('\n') {
                    let part = part.trim();
                    if part.is_empty() {
                        writeln!(f, "{part}")?;
                    } else {
                        writeln!(f, "{next_i}{}", part.trim_end())?;
                    }
                }

                writeln!(f, "{i}</Code>")?;
            }
            NodeKind::CodeSelect => {
                writeln!(f, "{i}<CodeSelect>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</CodeSelect>")?;
            }
            NodeKind::Heading { level, .. } => {
                writeln!(f, "{i}<Heading{level}>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</Heading{level}>")?;
            }
            NodeKind::TableRow => {
                writeln!(f, "{i}<TableRow>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</TableRow>")?;
            }
            NodeKind::TableCell => {
                writeln!(f, "{i}<TableCell>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</TableCell>")?;
            }
            NodeKind::Grid(grid) => {
                write!(f, "{i}<Grid")?;
                write!(f, " gap={{{}}}", grid.gap)?;
                write!(f, " columns={{{}}}", grid.columns)?;
                writeln!(f, ">")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Grid>")?;
            }
            NodeKind::BlockQuote => {
                writeln!(f, "{i}<BlockQuote>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</BlockQuote>")?;
            }
            NodeKind::ThematicBreak => {
                writeln!(f, "{i}<ThematicBreak />")?;
            }
            NodeKind::Break => {
                writeln!(f, "{i}<Break />")?;
            }
            NodeKind::Emphasis => {
                writeln!(f, "{i}<Emphasis>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</Emphasis>")?;
            }
            NodeKind::Strong => {
                writeln!(f, "{i}<Strong>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</Strong>")?;
            }
            NodeKind::Box(CBox {
                padding,
                max_width,
                class,
                height,
            }) => {
                write!(f, "{i}<Box")?;

                write!(f, " padding={{{padding}}}")?;
                write!(f, " max_width={{{max_width}}}")?;
                write!(f, " class={{{class}}}")?;
                write!(f, " height={{{:?}}}", height)?;

                writeln!(f, ">")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</Box>")?;
            }
            NodeKind::Delete => {
                writeln!(f, "{i}<Delete>")?;

                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</Delete>")?;
            }
            NodeKind::ListItem { checked, .. } => {
                write!(f, "{i}<ListItem")?;

                if let Some(checked) = checked {
                    write!(f, " checked={{{checked}}}")?;
                }

                writeln!(f, ">")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</ListItem>")?;
            }
            NodeKind::List { ordered, .. } => {
                write!(f, "{i}<List")?;

                if *ordered {
                    write!(f, " ordered")?;
                }

                writeln!(f, ">")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</List>")?;
            }
            NodeKind::Text { value } => {
                writeln!(f, "{i}<Text>")?;
                for part in value.split('\n') {
                    let part = part.trim();
                    if part.is_empty() {
                        writeln!(f, "{part}")?;
                    } else {
                        writeln!(f, "{next_i}{}", part.trim_end())?;
                    }
                }
                writeln!(f, "{i}</Text>")?;
            }
            NodeKind::InlineCode { value } => {
                writeln!(f, "{i}<InlineCode>")?;
                for part in value.split('\n') {
                    let part = part.trim();
                    if part.is_empty() {
                        writeln!(f, "{part}")?;
                    } else {
                        writeln!(f, "{next_i}{}", part.trim_end())?;
                    }
                }
                writeln!(f, "{i}</InlineCode>")?;
            }
            NodeKind::HtmlTag { value } => {
                for part in value.split('\n') {
                    let part = part.trim();
                    if part.is_empty() {
                        writeln!(f, "{part}")?;
                    } else {
                        writeln!(f, "{i}{}", part.trim_end())?;
                    }
                }
            }
            NodeKind::HtmlBlock { name, attributes } => {
                write!(f, "{i}<{name}")?;
                for attr in attributes {
                    let key = &attr.key;
                    write!(f, " {key}")?;
                    if let Some(value) = &attr.value {
                        write!(f, "={{{}}}", value.as_str())?;
                    }
                }
                writeln!(f, ">")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</{name}>")?;
            }
            NodeKind::Link { url, title } => {
                write!(f, "{i}<Link ")?;
                write!(f, "url={{{url}}}")?;
                if let Some(title) = title {
                    write!(f, " title={{{title}}}")?;
                }
                writeln!(f, ">")?;
                for child in &self.children {
                    child._debug_string(indent + 1, f)?;
                }
                writeln!(f, "{i}</Link>")?;
            }
            NodeKind::OpenAPISchema(schema) => {
                write!(f, "{i}<OpenAPISchema")?;

                if let Some(title) = &schema.title {
                    write!(f, " title={{{title}}}")?;
                }

                write!(f, " expanded={{{}}}", &schema.expanded)?;

                writeln!(f, ">")?;
                if let Some(node) = &schema.description_ast {
                    node._debug_string(indent + 1, f)?;
                }

                writeln!(f, "{i}</OpenAPISchema>")?;
            }
        }

        Ok(())
    }
}

#[derive(Debug, Default, Clone, Serialize)]
#[serde(tag = "name", content = "data", rename_all = "snake_case")]
pub enum NodeKind {
    #[default]
    Root,
    BlockQuote,
    ThematicBreak,
    Break,
    Strong,
    Emphasis,
    Delete,
    Math {
        value: String,
        display_mode: bool,
    },
    InlineMath {
        value: String,
    },
    Link {
        url: String,
        title: Option<String>,
    },
    Image {
        url: String,
        alt: String,
        title: Option<String>,
    },
    List {
        ordered: bool,
        start: Option<u32>,
        spread: bool,
    },
    ListItem {
        checked: Option<bool>,
        spread: bool,
    },
    Code {
        value: String,
        rendered_value: Option<String>,
        language: Option<String>,
        title: Option<String>,
        label: Option<String>,
        raw: bool,
        show_whitespace: bool,
    },
    InlineCode {
        value: String,
    },
    Text {
        value: String,
    },
    Heading {
        level: u8,
        slug: String,
    },
    Paragraph,
    /// A block-level HTML node. E.g.
    ///
    /// <div>     <-
    ///   ...        \
    ///   ...         |- this whole block
    ///   ...        /
    /// </div>    <-
    ///
    /// Only parsed in MDX mode
    HtmlBlock {
        name: String,
        attributes: Vec<Attribute>,
    },
    /// A single HTML tag that has been parsed, without any
    /// understanding of nested children.
    ///
    /// <div>foo</div>
    /// ^^^^^
    ///   |
    ///    \
    ///      - Just this single tag
    ///
    /// Only parsed in GFM mode.
    ///
    HtmlTag {
        value: String,
    },
    TableCell,
    TableRow,
    Table {
        alignment: Vec<TableAlignment>,
    },

    // ---------START Custom components -------------
    Grid(Grid),
    Flex(Flex),
    Box(CBox),
    #[serde(rename = "md_tabs")]
    Tabs,
    #[serde(rename = "md_tab")]
    Tab(Tab),
    Steps,
    Step(Step),
    CodeSelect,
    // ----------END Custom components --------------

    // ----------START OpenAPI Components -----------
    #[serde(rename = "open_api_schema")]
    OpenAPISchema(SchemaAst),
    // ----------END OpenAPI Components -----------
}

impl Node {
    /// Recursively find all the text from this node. Adds spaces between each piece of text.
    pub fn inner_text(&self) -> String {
        let mut out = String::new();

        for node in self.walk() {
            match &node.kind {
                NodeKind::Text { ref value } => {
                    out.push_str(value);
                }
                NodeKind::Break | NodeKind::Paragraph => {
                    out.push(' ');
                }
                NodeKind::Code { value, .. } => {
                    out.push(' ');
                    out.push_str(value);
                }
                NodeKind::InlineCode { value, .. } => {
                    out.push_str(value);
                }
                _ => {}
            }
        }

        out.trim().to_owned()
    }

    /// Traverses the children of this node in a depth-first manner
    pub fn walk(&self) -> impl Iterator<Item = &Node> {
        pub struct Descendants<'a> {
            stack: Vec<&'a Node>,
        }

        impl<'a> Iterator for Descendants<'a> {
            type Item = &'a Node;

            fn next(&mut self) -> Option<Self::Item> {
                self.stack.pop().inspect(|node| {
                    self.stack.extend(node.children.iter().rev());
                })
            }
        }

        Descendants { stack: vec![self] }
    }

    pub fn statistics(&self) -> ProseStatistics {
        let mut stats = ProseStatistics::default();

        // Count root level headings and paragraphs
        for node in self.children.iter() {
            match &node.kind {
                NodeKind::Heading { .. } => {
                    stats.headings += 1;
                }
                NodeKind::Paragraph => {
                    stats.paragraphs += 1;
                }
                _ => {}
            }
        }

        // Count words across all nodes
        for node in self.walk() {
            if let NodeKind::Text { ref value } = node.kind {
                stats.words += value.split_whitespace().count() as u32;
            }
        }

        stats
    }
}

#[derive(Debug, Default, Clone, Serialize)]
pub struct ProseStatistics {
    pub headings: u32,
    pub paragraphs: u32,
    pub words: u32,
}

#[cfg(test)]
mod test {
    use crate::ast_mdx;
    use crate::markdown::ast;
    use crate::render_context::RenderContext;

    #[test]
    fn inner_text() {
        let markdown = indoc! {r#"
        Foo **bar** _baz_ [fizz](example.com).
        "#};

        let ctx = RenderContext::new();
        let ast = ast(markdown, &ctx).unwrap();

        assert_eq!(ast.inner_text(), "Foo bar baz fizz.");
    }

    #[test]
    fn inner_text_breaks() {
        let markdown = indoc! {r#"
        Foo\
        bar\
        baz
        "#};

        let ctx = RenderContext::new();
        let ast = ast(markdown, &ctx).unwrap();

        assert_eq!(ast.inner_text(), "Foo bar baz");
    }

    #[test]
    fn inner_text_code() {
        let markdown = indoc! {r#"
        `inline` code

        ```
        block code
        ```
        "#};

        let ctx = RenderContext::new();
        let ast = ast(markdown, &ctx).unwrap();

        assert_eq!(ast.inner_text(), "inline code block code");
    }

    #[test]
    fn statistics() {
        let markdown = indoc! {r#"
        # Foo

        ## Bar

        These are some words.

        <Box>
          # Ignored header
        </Box>
        "#};

        let ctx = RenderContext::new();
        let ast = ast_mdx(markdown, &ctx).unwrap();

        let stats = ast.statistics();

        assert_eq!(stats.headings, 2);
        assert_eq!(stats.paragraphs, 1);
        assert_eq!(stats.words, 8);
    }
}
