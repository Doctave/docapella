use std::str::FromStr;

use super::{markdown_rs_error_wrapper, shared_ast::*};
use markdown_rs::mdast::{self, InlineMath, Math, MdxFlowExpression, MdxTextExpression};

use crate::control_flow::conditional::{fold_conditionals, Conditional, Operation};
use crate::primitive_components::Primitive;
use crate::{render_context::RenderContext, Error, Result};

pub(crate) fn build_mdx(markdown_input: &str, ctx: &RenderContext) -> Result<Node> {
    let mut opts = markdown_rs::Options::default();
    let mut parse_opts = markdown_rs::ParseOptions::mdx();
    parse_opts.constructs.mdx_esm = false;
    parse_opts.constructs.gfm_table = true;
    parse_opts.constructs.gfm_task_list_item = true;
    parse_opts.constructs.gfm_autolink_literal = false;
    parse_opts.constructs.gfm_footnote_definition = false;
    parse_opts.constructs.gfm_label_start_footnote = false;
    parse_opts.constructs.math_flow = true;
    parse_opts.constructs.math_text = true;

    opts.parse = parse_opts;

    // Parse the markdown file into an AST
    markdown_rs::to_mdast(markdown_input, &opts.parse)
        .map(|n| Node::from_mdast(n, markdown_input, ctx))
        .map_err(|e| Error {
            code: Error::INVALID_MARKDOWN_TEMPLATE,
            message: "Unable to parse Markdown template".to_string(),
            description: markdown_rs_error_wrapper::pretty_error_msg(&e, markdown_input, ctx),
            file: None,
            position: markdown_rs_error_wrapper::parse_position(e, markdown_input)
                .as_ref()
                .map(|p| (&**p).into()),
        })?
}

pub(crate) fn build_gfm(markdown_input: &str, ctx: &RenderContext) -> Result<Node> {
    let mut opts = markdown_rs::Options::gfm();
    opts.parse.constructs.gfm_autolink_literal = false;
    opts.parse.constructs.gfm_footnote_definition = false;
    opts.parse.constructs.gfm_label_start_footnote = false;

    // Parse the markdown file into an AST
    markdown_rs::to_mdast(markdown_input, &opts.parse)
        .map(|n| Node::from_mdast(n, markdown_input, ctx))
        .map_err(|e| Error {
            code: Error::INVALID_MARKDOWN_TEMPLATE,
            message: "Unable to parse Markdown template".to_string(),
            description: markdown_rs_error_wrapper::pretty_error_msg(&e, markdown_input, ctx),
            file: None,
            position: markdown_rs_error_wrapper::parse_position(e, markdown_input)
                .as_ref()
                .map(|p| (&**p).into()),
        })?
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub kind: NodeKind,
    pub pos: Position,
    pub children: Vec<Node>,
}

impl Node {
    /// Recursively find all the text from this node. Adds spaces between each piece of text.
    #[allow(dead_code)]
    pub fn inner_text(&self) -> String {
        let mut out = vec![];

        fn traverse<'a>(node: &'a Node, out: &mut Vec<&'a str>) {
            match &node.kind {
                NodeKind::Text { ref value } => {
                    if !value.trim().is_empty() {
                        out.push(value.trim());
                    }
                }
                NodeKind::Code { value, .. } => {
                    if !value.trim().is_empty() {
                        out.push(value.trim());
                    }
                }
                NodeKind::InlineCode { value, .. } => {
                    if !value.trim().is_empty() {
                        out.push(value.trim());
                    }
                }
                _ => {}
            }

            for child in &node.children {
                traverse(child, out);
            }
        }

        traverse(self, &mut out);

        out.join(" ")
    }

    /// Traverses the children of this node in a depth-first manner
    #[allow(dead_code)]
    pub fn descendants(&self) -> impl Iterator<Item = &Node> {
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

    pub fn is_whitespace_or_linebreak(&self) -> bool {
        if let NodeKind::Text { value } = &self.kind {
            value.as_str() == "\n" || value.as_str() == " "
        } else {
            false
        }
    }

    pub fn is_start_of_sequence(&self) -> bool {
        if let NodeKind::Conditional(cond) = &self.kind {
            cond.op == Operation::If
        } else {
            false
        }
    }

    pub fn is_conditional(&self) -> bool {
        matches!(self.kind, NodeKind::Conditional(_))
    }

    pub fn wrap_with_conditional(self, op: Operation, cond_expr: Option<String>) -> Self {
        let kind = NodeKind::Conditional(Conditional {
            cond_expr,
            op,
            true_branch: Box::new(self),
            false_branch: Box::new(Node {
                pos: Position::default(),
                kind: NodeKind::Noop,
                children: vec![],
            }),
        });

        Node {
            pos: Position::default(),
            kind,
            children: vec![],
        }
    }
}

impl Node {
    fn render_children(
        other: &mut mdast::Node,
        src: &str,
        ctx: &RenderContext,
    ) -> Result<Vec<Node>> {
        let mut children = other
            .children_mut()
            .map(|children| {
                children
                    .drain(..)
                    .map(|c| Node::from_mdast(c, src, ctx))
                    .collect()
            })
            .unwrap_or(Ok(vec![]))?;

        let any_conditionals_in_children = children.iter().any(|c| c.is_conditional());

        if any_conditionals_in_children {
            children = fold_conditionals(children).map_err(|e| Error {
                code: Error::INVALID_CONDITIONAL,
                message: "Error in condition".to_string(),
                description: e.render(src, ctx),
                file: None,
                position: Some(e.position()),
            })?;
        }

        Ok(children)
    }

    fn from_mdast(mut other: mdast::Node, src: &str, ctx: &RenderContext) -> Result<Self> {
        let mut children = Node::render_children(&mut other, src, ctx)?;

        let pos = other.position().map(|p| p.into()).unwrap_or_default();

        let mut conditional_operators = None;

        let kind = match other {
            mdast::Node::Root(_) => NodeKind::Root,
            mdast::Node::Paragraph(_) => {
                // NOTE: Context for this transformation in DOC-1136
                //
                // https://github.com/wooorm/markdown-rs/issues/116
                // https://linear.app/doctave/issue/DOC-1136/extra-paragraph-tags-inserted-with-using-a-raw-p-tag
                // https://github.com/wooorm/mdxjs-rs/blob/b1971be2dbdd2886ca4d5e9d97d9a3477cb29904/src/mdast_util_to_hast.rs#L925
                let mut all_mdx = true;
                let mut one_or_more_mdx = false;

                for child in &children {
                    match child.kind {
                        NodeKind::Text { ref value } => {
                            if value.trim().is_empty() {
                                continue;
                            } else {
                                all_mdx = false;
                                break;
                            }
                        }
                        ref n => {
                            if n.is_doctave_primitive_component()
                                || matches!(
                                    n,
                                    NodeKind::Component { .. } | NodeKind::HtmlBlock { .. }
                                )
                            {
                                one_or_more_mdx = true;
                            } else {
                                all_mdx = false;
                                break;
                            }
                        }
                    }
                }

                if all_mdx && one_or_more_mdx {
                    NodeKind::Root // Return as root to indicate unwrapped content
                } else {
                    NodeKind::Paragraph
                }
            }
            mdast::Node::Text(mdast::Text { value, .. }) => NodeKind::Text { value },
            mdast::Node::Heading(mdast::Heading { depth: level, .. }) => {
                NodeKind::Heading { level }
            }
            mdast::Node::ThematicBreak(_) => NodeKind::ThematicBreak,
            mdast::Node::Break(_) => NodeKind::Break,
            mdast::Node::Strong(_) => NodeKind::Strong,
            mdast::Node::Emphasis(_) => NodeKind::Emphasis,
            mdast::Node::Delete(_) => NodeKind::Delete,
            mdast::Node::Blockquote(_) => NodeKind::BlockQuote,
            mdast::Node::Definition(mdast::Definition {
                url,
                title,
                identifier,
                label,
                ..
            }) => NodeKind::Definition {
                url,
                title,
                identifier,
                label,
            },
            mdast::Node::Link(mdast::Link { url, title, .. }) => NodeKind::Link { url, title },
            mdast::Node::LinkReference(mdast::LinkReference {
                identifier,
                label,
                reference_kind,
                ..
            }) => NodeKind::LinkReference {
                identifier,
                label,
                reference_kind: reference_kind.into(),
            },
            mdast::Node::Image(mdast::Image {
                url, title, alt, ..
            }) => NodeKind::Image { url, title, alt },
            mdast::Node::ImageReference(mdast::ImageReference {
                identifier,
                label,
                reference_kind,
                alt,
                ..
            }) => NodeKind::ImageReference {
                identifier,
                label,
                reference_kind: reference_kind.into(),
                alt,
            },
            mdast::Node::Code(mdast::Code {
                value, lang, meta, ..
            }) => NodeKind::Code {
                value,
                language: lang,
                meta,
            },
            mdast::Node::InlineCode(mdast::InlineCode { value, .. }) => {
                NodeKind::InlineCode { value }
            }
            mdast::Node::TableCell(_) => NodeKind::TableCell,
            mdast::Node::TableRow(_) => NodeKind::TableRow,
            mdast::Node::Table(mdast::Table { align, .. }) => NodeKind::Table {
                alignment: align.into_iter().map(|a| a.into()).collect(),
            },
            mdast::Node::List(mdast::List {
                ordered,
                start,
                spread,
                ..
            }) => NodeKind::List {
                start,
                ordered,
                spread,
            },
            mdast::Node::ListItem(mdast::ListItem {
                checked, spread, ..
            }) => NodeKind::ListItem { checked, spread },
            // These MdxJsxFlowElement nodes will never show up in this current release, but I'm
            // including this code for now since it's already written.
            mdast::Node::MdxJsxTextElement(mdast::MdxJsxTextElement {
                name, attributes, ..
            })
            | mdast::Node::MdxJsxFlowElement(mdast::MdxJsxFlowElement {
                name, attributes, ..
            }) => {
                let attributes: Vec<Attribute> = attributes.into_iter().map(|a| a.into()).collect();

                conditional_operators = attributes
                    .iter()
                    .find(|attr| Operation::from_str(&attr.key).is_ok())
                    .and_then(|attr| {
                        let val = attr.value.as_ref().map(|v| v.as_str().to_string());
                        Operation::from_str(&attr.key).ok().map(|o| (o, val))
                    });

                let name = name.unwrap_or("Unknown-node".to_owned());
                if name
                    .chars()
                    .next()
                    .map(|c| c.is_uppercase())
                    .unwrap_or(false)
                {
                    if let Some(primitive) = Primitive::parse_from_str(&name) {
                        primitive
                            .try_into_content_node_kind(attributes, &pos)
                            .map_err(|e| Error {
                                code: Error::INVALID_COMPONENT,
                                message: "Unexpected attribute".to_string(),
                                description: e.render(src, ctx),
                                file: None,
                                position: Some(pos.clone()),
                            })?
                    } else {
                        NodeKind::Component { name, attributes }
                    }
                } else {
                    NodeKind::HtmlBlock { name, attributes }
                }
            }
            mdast::Node::Html(mdast::Html { value, .. }) => NodeKind::HtmlTag { value },
            // These notes will never show up because they've been disabled in the markdown_rs
            // parser options entirely. There are tests for each of these nodes to ensure the
            // inputs don't tigger the parser to create these nodes.
            mdast::Node::FootnoteDefinition(_) => {
                unimplemented!("FootnoteDefinition not supported")
            }
            mdast::Node::MdxFlowExpression(MdxFlowExpression { value, .. }) => {
                NodeKind::ExpressionBlock { value }
            }
            mdast::Node::MdxTextExpression(MdxTextExpression { value, .. }) => {
                NodeKind::Expression { value }
            }
            mdast::Node::FootnoteReference(_) => unimplemented!("FootnoteReference not supported"),
            mdast::Node::MdxjsEsm(_) => unimplemented!("MdxjsEsm (import tag) not supported"),
            mdast::Node::Toml(_) => unimplemented!("Toml not supported"),
            mdast::Node::Yaml(_) => unimplemented!("Yaml  not supported"),
            mdast::Node::Math(Math { value, meta, .. }) => NodeKind::Math { value, meta },
            mdast::Node::InlineMath(InlineMath { value, .. }) => NodeKind::InlineMath { value },
        };

        ensure_balanced_tables(&kind, &mut children);

        let node = Node {
            children,
            pos,
            kind,
        };

        if let Some((op, cond_expr)) = conditional_operators {
            Ok(node.wrap_with_conditional(op, cond_expr))
        } else {
            Ok(node)
        }
    }
}

/// Ensure that a table has the correct number of columns in each row.
///
/// This function will either drop or add the appropriate number of columns
/// based on the table's alignment.
fn ensure_balanced_tables(kind: &NodeKind, children: &mut Vec<Node>) {
    if let NodeKind::Table { alignment } = kind {
        for row in children {
            // First check we weren't short any cells
            while row.children.len() < alignment.len() {
                row.children.push(Node {
                    kind: NodeKind::TableCell,
                    pos: row.pos.clone(),
                    children: vec![],
                })
            }

            // Then check that we weren't too long.
            // This is a no-op if the first part added any cells
            row.children.drain(alignment.len()..);
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum ReferenceKind {
    Shortcut,
    Collapsed,
    Full,
}

impl From<mdast::ReferenceKind> for ReferenceKind {
    fn from(value: mdast::ReferenceKind) -> Self {
        match value {
            mdast::ReferenceKind::Full => ReferenceKind::Full,
            mdast::ReferenceKind::Shortcut => ReferenceKind::Shortcut,
            mdast::ReferenceKind::Collapsed => ReferenceKind::Collapsed,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum NodeKind {
    Root,
    BlockQuote,
    ThematicBreak,
    Break,
    Strong,
    Emphasis,
    Delete,
    Math {
        value: String,
        meta: Option<String>,
    },
    InlineMath {
        value: String,
    },
    Link {
        url: String,
        title: Option<String>,
    },
    LinkReference {
        identifier: String,
        label: Option<String>,
        reference_kind: ReferenceKind,
    },
    Image {
        url: String,
        alt: String,
        title: Option<String>,
    },
    ImageReference {
        identifier: String,
        label: Option<String>,
        reference_kind: ReferenceKind,
        alt: String,
    },
    Definition {
        url: String,
        title: Option<String>,
        identifier: String,
        label: Option<String>,
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
        language: Option<String>,
        meta: Option<String>,
    },
    InlineCode {
        value: String,
    },
    Text {
        value: String,
    },
    Heading {
        level: u8,
    },
    Paragraph,
    Component {
        name: String,
        attributes: Vec<Attribute>,
    },
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
    Expression {
        value: String,
    },
    ExpressionBlock {
        value: String,
    },
    TableCell,
    TableRow,
    Table {
        alignment: Vec<TableAlignment>,
    },

    /* CONTROL FLOW */
    Conditional(Conditional),
    Noop,

    /* PRIMITIVES */
    Tabs,
    Tab {
        title: Option<AttributeValue>,
    },
    Steps,
    Step {
        title: Option<AttributeValue>,
    },
    CodeSelect {
        title: Option<AttributeValue>,
    },
    Flex {
        align: Option<AttributeValue>,
        justify: Option<AttributeValue>,
        direction: Option<AttributeValue>,
        wrap: Option<AttributeValue>,
        gap: Option<AttributeValue>,
        padding: Option<AttributeValue>,
        height: Option<AttributeValue>,
        class: Option<AttributeValue>,
    },
    Box {
        max_width: Option<AttributeValue>,
        class: Option<AttributeValue>,
        padding: Option<AttributeValue>,
        height: Option<AttributeValue>,
    },
    Grid {
        cols: Option<AttributeValue>,
        gap: Option<AttributeValue>,
    },
    Slot,

    // OPENAPI
    OpenAPISchema {
        openapi_path: Option<AttributeValue>,
        title: Option<AttributeValue>,
        expanded: Option<AttributeValue>,
    },
}

impl NodeKind {
    pub(crate) fn is_doctave_primitive_component(&self) -> bool {
        use NodeKind::*;

        // NOTE: (Nik) I'm purposefully writing out all possible variants here because this would
        // be super easy to forget to update if we added a catch all.
        match self {
            Root => false,
            BlockQuote => false,
            ThematicBreak => false,
            Break => false,
            Strong => false,
            Emphasis => false,
            Delete => false,
            Math { .. } => false,
            InlineMath { .. } => false,
            Link { .. } => false,
            LinkReference { .. } => false,
            Image { .. } => false,
            ImageReference { .. } => false,
            Definition { .. } => false,
            List { .. } => false,
            ListItem { .. } => false,
            Code { .. } => false,
            InlineCode { .. } => false,
            Text { .. } => false,
            Heading { .. } => false,
            Paragraph => false,
            Component { .. } => false,
            HtmlBlock { .. } => false,
            HtmlTag { .. } => false,
            Expression { .. } => false,
            ExpressionBlock { .. } => false,
            TableCell => false,
            TableRow => false,
            Table { .. } => false,
            Conditional(_) => false,
            Noop => false,
            Tabs => true,
            Tab { .. } => true,
            Steps => true,
            Step { .. } => true,
            CodeSelect { .. } => true,
            Flex { .. } => true,
            Box { .. } => true,
            Grid { .. } => true,
            OpenAPISchema { .. } => true,
            Slot => true,
            // DON'T ADD A CATCH ALL!
            // We want the compiler to warn us to update this list if a new primitive is added
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use indoc::indoc;

    #[test]
    fn inner_text() {
        let input = indoc! {r#"
        # Foo **bar** _baz_ `fizz`
        "#};

        let root = build_gfm(input, &RenderContext::new()).unwrap();

        assert_eq!(root.inner_text(), "Foo bar baz fizz");
    }
}
