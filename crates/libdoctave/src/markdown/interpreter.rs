use rust_decimal::Decimal;

use crate::{
    attribute_parser::parse_attributes,
    content_ast::{Node as ContentNode, NodeKind as ContentNodeKind},
    control_flow::conditional::Conditional,
    expressions::{Environment, Value},
    markdown::{
        custom_components::custom_component::Error as ComponentError,
        expressions::Interpreter as ExprInterpreter, sanitizer::SANITIZER,
    },
    open_api::ast::SchemaAst,
    primitive_components::{CBox, CodeSelect, Flex, Grid, Step, Steps, Tab, Tabs},
    render_context::{FileContext, RenderContext},
    renderable_ast::{Node, NodeKind, Position},
    utils::capitalize,
    Anchorizer, AttributeValue, CustomComponent, Error,
};
/// The interpreter that turns a Content AST that describes _what_ is in the
/// document into a renderable AST.
///
/// This means:
/// - Resolving and verifying all component invocations
/// - Finding and rendering all partials
/// - Evaluating any expressions
use crate::{Attribute, Result};

use std::{collections::HashMap, str::FromStr};

use super::{
    custom_components::custom_component::ComponentKind,
    error_renderer::{self, Highlight, Location},
    expressions,
};

pub(crate) struct Interpreter<'a> {
    ctx: &'a RenderContext<'a>,
    state: ConversionState,
    input: &'a str,
    pub expr_interpreter: ExprInterpreter,
    pub slot_injection: Option<Vec<Node>>,
    pub stack_depth: u8,
}

impl<'a> Interpreter<'a> {
    pub fn new(ctx: &'a RenderContext, input: &'a str) -> Self {
        let expr_interpreter = ExprInterpreter::new(Some(Environment::default()));

        Interpreter {
            ctx,
            state: ConversionState::default(),
            input,
            expr_interpreter,
            slot_injection: None,
            stack_depth: 0,
        }
    }

    pub fn interpret(&mut self, content_ast: ContentNode) -> Result<Node> {
        self.find_references(&content_ast);

        if let Some(root) = self.walk(content_ast)? {
            Ok(root)
        } else {
            unreachable!("BUG: Root of AST was not a `Root` node")
        }
    }

    fn walk(&mut self, content_ast: ContentNode) -> Result<Option<Node>> {
        let ContentNode {
            pos,
            children,
            kind,
            ..
        } = content_ast;

        match kind {
            ContentNodeKind::Noop => Ok(None),
            ContentNodeKind::Root => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Root,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::BlockQuote => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::BlockQuote,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Break => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Break,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::ThematicBreak => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::ThematicBreak,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Strong => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Strong,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Emphasis => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Emphasis,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Delete => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Delete,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Link { url, title } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Link { url, title },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::LinkReference { identifier, .. } => {
                let children = self.render_children(children)?;

                if let Some(Reference { url, title, .. }) = self.state.definitions.get(&identifier)
                {
                    Ok(Some(Node {
                        kind: NodeKind::Link {
                            url: url.clone(),
                            title: title.clone(),
                        },
                        children,
                        pos,
                    }))
                } else {
                    Ok(Some(Node {
                        kind: NodeKind::Link {
                            url: "".to_string(),
                            title: None,
                        },
                        children,
                        pos,
                    }))
                }
            }
            ContentNodeKind::Image { url, title, alt } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Image { url, title, alt },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Tab { title } => {
                let children = self.render_children(children)?;

                let title = self.evaluate_option_value(title, &pos)?;

                let tab = Tab::try_new(title).map_err(|e| Error {
                    code: Error::INVALID_COMPONENT,
                    message: "Error in tab".to_string(),
                    description: e.render(self.input, self.ctx, &pos),
                    file: None,
                    position: None,
                })?;

                let kind = NodeKind::Tab(tab);

                Ok(Some(Node {
                    kind,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Tabs => {
                let mut children = self.render_children(children)?;

                if children.len() == 1
                    && matches!(children[0].kind, NodeKind::Paragraph | NodeKind::Root)
                {
                    children = std::mem::take(&mut children[0].children);
                }

                for next in &children {
                    Tabs::verify(next).map_err(|e| Error {
                        code: Error::INVALID_TABS,
                        message: "Error in tabs".to_string(),
                        description: e.render(self.input, self.ctx, &next.pos),
                        file: None,
                        position: None,
                    })?;
                }

                let component = Node {
                    kind: NodeKind::Tabs,
                    children,
                    pos,
                };

                Ok(Some(component))
            }
            ContentNodeKind::Steps => {
                let mut children = self.render_children(children)?;

                if children.len() == 1
                    && matches!(children[0].kind, NodeKind::Paragraph | NodeKind::Root)
                {
                    children = std::mem::take(&mut children[0].children);
                }

                for next in &children {
                    Steps::verify(next).map_err(|e| Error {
                        code: Error::INVALID_TABS,
                        message: "Error in steps".to_string(),
                        description: e.render(self.input, self.ctx, &next.pos),
                        file: None,
                        position: None,
                    })?;
                }

                let steps = Node {
                    kind: NodeKind::Steps,
                    children,
                    pos,
                };

                Ok(Some(steps))
            }
            ContentNodeKind::Step { title } => {
                let children = self.render_children(children)?;

                let title = self.evaluate_option_value(title, &pos)?;

                let step = Step::try_new(title).map_err(|e| Error {
                    code: Error::INVALID_STEPS,
                    message: "Error in step".to_string(),
                    description: e.render(self.input, self.ctx, &pos),
                    file: None,
                    position: None,
                })?;

                let kind = NodeKind::Step(step);

                Ok(Some(Node {
                    kind,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::ImageReference {
                identifier, alt, ..
            } => {
                let children = self.render_children(children)?;

                if let Some(Reference { url, title, .. }) = self.state.definitions.get(&identifier)
                {
                    Ok(Some(Node {
                        kind: NodeKind::Image {
                            url: url.clone(),
                            title: title.clone(),
                            alt,
                        },
                        children,
                        pos,
                    }))
                } else {
                    Ok(Some(Node {
                        kind: NodeKind::Image {
                            url: "".to_string(),
                            title: None,
                            alt,
                        },
                        children,
                        pos,
                    }))
                }
            }
            ContentNodeKind::List {
                ordered,
                start,
                spread,
            } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::List {
                        ordered,
                        start,
                        spread,
                    },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::ListItem { checked, spread } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::ListItem { checked, spread },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Code {
                value,
                language,
                meta,
            } => {
                let children = self.render_children(children)?;

                let mut title = None;
                let mut label = language.clone().map(|l| capitalize(&l));
                let mut raw = false;
                let mut show_whitespace = false;

                if let Some(meta) = &meta {
                    let mut attrs = parse_attributes(meta);

                    if let Some(t) = attrs.remove("title") {
                        title = t;
                    }

                    if let Some(l) = attrs.remove("label") {
                        label = l;
                    }

                    if let Some(r) = attrs.remove("raw") {
                        raw = r.unwrap_or("true".to_string()).parse().unwrap_or(false);
                    }

                    if let Some(sw) = attrs.remove("show-whitespace") {
                        show_whitespace = sw.unwrap_or("true".to_string()).parse().unwrap_or(false);
                    }
                }

                Ok(Some(Node {
                    kind: NodeKind::Code {
                        value,
                        language,
                        title,
                        label,
                        raw,
                        show_whitespace,
                        rendered_value: None,
                    },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::CodeSelect { title } => {
                let mut children = self.render_children(children)?;

                let title = self.evaluate_option_value(title, &pos)?;

                CodeSelect::overwrite_code_blocks(&mut children, title.as_ref());

                CodeSelect::verify_code_children(&children).map_err(|e| Error {
                    code: Error::INVALID_COMPONENT,
                    message: "Error in code tabs".to_string(),
                    description: e.render(self.input, self.ctx, &pos),
                    file: None,
                    position: None,
                })?;

                // Unwrap lone codetabs as regular code block
                if children.len() == 1 {
                    Ok(Some(children.remove(0)))
                } else {
                    Ok(Some(Node {
                        kind: NodeKind::CodeSelect,
                        children,
                        pos,
                    }))
                }
            }
            ContentNodeKind::InlineCode { value } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::InlineCode { value },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Text { value } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Text { value },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Heading { level } => {
                let children = self.render_children(children)?;

                // Construct the node with a fake slug so that we get access ot the `inner_text`
                // function, which will make our life a lot simpler.
                let mut node = Node {
                    kind: NodeKind::Heading {
                        level,
                        slug: String::new(),
                    },
                    children,
                    pos,
                };

                let anchorized_slug = self.state.anchorizer.anchorize(node.inner_text());

                if let NodeKind::Heading { ref mut slug, .. } = &mut node.kind {
                    *slug = anchorized_slug;
                }

                Ok(Some(node))
            }
            ContentNodeKind::Paragraph => {
                let children = self.render_children(children)?;

                if children.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(Node {
                        kind: NodeKind::Paragraph,
                        children,
                        pos,
                    }))
                }
            }
            ContentNodeKind::Component { name, attributes } => {
                if let Some(handle) = self
                    .ctx
                    .custom_components
                    .iter()
                    .find(|c| c.matches_title(&name))
                {
                    // Validate recursion
                    if self.stack_depth > 50 {
                        return Err(Error {
                            code: Error::INVALID_COMPONENT,
                            message: "Recursive component call".to_string(),
                            description: ComponentError::RecursiveComponent(name, pos.clone())
                                .render(self.input, self.ctx),
                            file: self.ctx.file_context.as_ref().map(|f| f.fs_path.clone()),
                            position: None,
                        });
                    }

                    // Render the custom component
                    //
                    // 1. Match the attributes
                    // 2. Interpret the component in an isolated environment with the attributes as
                    //    globals
                    // 3. Replace the node with the output of the rendered custom node
                    let c = handle.build().map_err(|e| Error {
                        code: Error::INVALID_COMPONENT,
                        message: e.to_string(),
                        description: e.render(&handle.content, self.ctx),
                        file: Some(handle.path.clone()),
                        position: None,
                    })?;

                    let attribute_values =
                        self.resolve_attributes(&c, attributes.as_slice(), &pos)?;

                    let mut slot_content = self.render_children(children)?;

                    if handle.unwrap_lone_p
                        && slot_content.len() == 1
                        && matches!(slot_content[0].kind, NodeKind::Paragraph)
                    {
                        let p_children = slot_content[0].children.drain(..);
                        slot_content = p_children.collect();
                    }

                    let component_ctx = RenderContext {
                        file_context: Some(FileContext::new(
                            handle.error_lines_offset(),
                            handle.error_bytes_offset(),
                            handle.path.clone(),
                        )),
                        ..self.ctx.clone()
                    };

                    let mut rendered_component = handle.evaluate(
                        &component_ctx,
                        attribute_values,
                        slot_content,
                        self.stack_depth,
                    )?;

                    rendered_component.pos = pos.clone();

                    Ok(Some(rendered_component))
                } else {
                    Err(Error {
                        code: Error::INVALID_COMPONENT,
                        message: format!("Unknown component {}", &name),
                        description: ComponentError::UnknownComponent(
                            ComponentKind::from_name(&name),
                            name,
                            pos.clone(),
                        )
                        .render(self.input, self.ctx),
                        file: self.ctx.file_context.as_ref().map(|f| f.fs_path.clone()),
                        position: None,
                    })
                }
            }
            ContentNodeKind::Slot => {
                if self.is_custom_component() {
                    Ok(Some(Node {
                        kind: NodeKind::Root,
                        children: self.slot_injection.take().unwrap_or_default(),
                        pos,
                    }))
                } else {
                    Err(Error {
                        code: Error::INVALID_COMPONENT,
                        message: String::from("Invalid slot"),
                        description: ComponentError::InvalidSlot(pos.clone())
                            .render(self.input, self.ctx),
                        file: self.ctx.file_context.as_ref().map(|f| f.fs_path.clone()),
                        position: None,
                    })
                }
            }
            ContentNodeKind::HtmlBlock { name, attributes } => {
                let children = self.render_children(children)?;
                let mut resolved_attributes = self.evaluate_attributes(attributes, &pos)?;

                if SANITIZER
                    .sanitize_html_block(&name, &mut resolved_attributes)
                    .is_none()
                {
                    return Ok(None);
                }

                Ok(Some(Node {
                    kind: NodeKind::HtmlBlock {
                        name,
                        attributes: resolved_attributes,
                    },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::HtmlTag { value } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::HtmlTag { value },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::TableCell => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::TableCell,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::TableRow => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::TableRow,
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Table { alignment } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::Table { alignment },
                    children,
                    pos,
                }))
            }
            ContentNodeKind::Definition { .. } => Ok(None),
            ContentNodeKind::Expression { ref value } => Ok(Some(Node {
                kind: NodeKind::Text {
                    value: self
                        .evaluate_expr(value.as_str())
                        .map(|val| val.to_string())
                        .map_err(|e| Error {
                            code: Error::INVALID_EXPRESSION,
                            message: "Error in expression".to_string(),
                            description: e.render(self.input, self.ctx, None, None, &pos),
                            file: None,
                            position: None,
                        })?,
                },
                children: vec![],
                pos,
            })),
            ContentNodeKind::ExpressionBlock { ref value } => Ok(Some(Node {
                kind: NodeKind::Paragraph,
                children: vec![Node {
                    kind: NodeKind::Text {
                        value: self
                            .evaluate_expr(value.as_str())
                            .map(|val| val.to_string())
                            .map_err(|e| Error {
                                code: Error::INVALID_EXPRESSION,
                                message: "Error in expression".to_string(),
                                description: e.render(self.input, self.ctx, None, None, &pos),
                                file: None,
                                position: None,
                            })?,
                    },
                    children: vec![],
                    pos: pos.clone(),
                }],
                pos,
            })),
            ContentNodeKind::Conditional(Conditional {
                true_branch,
                false_branch,
                cond_expr,
                ..
            }) => {
                // if there is no conditional expression, we should resolve to true
                let expr = cond_expr.unwrap_or("true".to_string());
                let val = self.evaluate_expr(&expr).map_err(|e| Error {
                    code: Error::INVALID_EXPRESSION,
                    message: "Error in expression".to_string(),
                    description: e.render(self.input, self.ctx, None, None, &pos),
                    file: None,
                    position: None,
                })?;

                if val.is_truthy() {
                    self.walk(*true_branch)
                } else {
                    self.walk(*false_branch)
                }
            }
            ContentNodeKind::Flex {
                align,
                justify,
                direction,
                wrap,
                gap,
                padding,
                height,
                class,
            } => {
                let children = self.render_children(children)?;

                let align = self.evaluate_option_value(align, &pos)?;
                let direction = self.evaluate_option_value(direction, &pos)?;
                let gap = self.evaluate_option_value(gap, &pos)?;
                let wrap = self.evaluate_option_value(wrap, &pos)?;
                let justify = self.evaluate_option_value(justify, &pos)?;
                let padding = self.evaluate_option_value(padding, &pos)?;
                let height = self.evaluate_option_value(height, &pos)?;
                let class = self.evaluate_option_value(class, &pos)?;

                let flex =
                    Flex::try_new(justify, align, direction, wrap, gap, padding, height, class)
                        .map_err(|e| Error {
                            code: Error::INVALID_COMPONENT,
                            message: "Error in flex".to_string(),
                            description: e.render(self.input, self.ctx, &pos),
                            file: None,
                            position: None,
                        })?;

                Ok(Some(Node {
                    kind: NodeKind::Flex(flex),
                    pos,
                    children,
                }))
            }
            ContentNodeKind::Box {
                max_width,
                padding,
                class,
                height,
            } => {
                let children = self.render_children(children)?;

                let max_width = self.evaluate_option_value(max_width, &pos)?;
                let class = self.evaluate_option_value(class, &pos)?;
                let padding = self.evaluate_option_value(padding, &pos)?;
                let height = self.evaluate_option_value(height, &pos)?;

                let c_box =
                    CBox::try_new(max_width, class, padding, height).map_err(|e| Error {
                        code: Error::INVALID_COMPONENT,
                        message: "Error in box".to_string(),
                        description: e.render(self.input, self.ctx, &pos),
                        file: None,
                        position: None,
                    })?;

                Ok(Some(Node {
                    kind: NodeKind::Box(c_box),
                    pos,
                    children,
                }))
            }
            ContentNodeKind::Grid { gap, cols } => {
                let children = self.render_children(children)?;

                let gap = self.evaluate_option_value(gap, &pos)?;
                let cols = self.evaluate_option_value(cols, &pos)?;

                let c_box = Grid::try_new(gap, cols).map_err(|e| Error {
                    code: Error::INVALID_COMPONENT,
                    message: "Error in grid".to_string(),
                    description: e.render(self.input, self.ctx, &pos),
                    file: None,
                    position: None,
                })?;

                Ok(Some(Node {
                    kind: NodeKind::Grid(c_box),
                    pos,
                    children,
                }))
            }
            ContentNodeKind::Math { value, meta } => {
                let children = self.render_children(children)?;

                let mut display_mode = false;

                if let Some(meta) = &meta {
                    let mut attrs = parse_attributes(meta);

                    let display_mode_attr = attrs
                        .remove("display_mode")
                        .unwrap_or(Some("false".to_string()))
                        .unwrap_or("false".to_string());

                    match display_mode_attr.as_str() {
                        "true" => display_mode = true,
                        _ => display_mode = false,
                    };
                }

                Ok(Some(Node {
                    kind: NodeKind::Math {
                        value,
                        display_mode,
                    },
                    pos,
                    children,
                }))
            }
            ContentNodeKind::InlineMath { value } => {
                let children = self.render_children(children)?;

                Ok(Some(Node {
                    kind: NodeKind::InlineMath { value },
                    pos,
                    children,
                }))
            }
            ContentNodeKind::OpenAPISchema {
                title,
                openapi_path,
                expanded,
            } => {
                let expanded = self
                    .evaluate_option_value(expanded, &pos)?
                    .map(|v| match v {
                        Value::Bool(b) => b,
                        Value::String(s) => s.parse().unwrap_or(true),
                        _ => true,
                    })
                    .unwrap_or(true);

                let openapi_path = self.evaluate_option_value(openapi_path, &pos)?;
                let title = self.evaluate_option_value(title, &pos)?;

                let schema =
                    SchemaAst::try_new_schema(title, openapi_path, self.ctx).map_err(|e| {
                        Error {
                            code: Error::INVALID_OPENAPI_SCHEMA,
                            message: "Error in openapi schema".to_string(),
                            description: e.render(self.input, self.ctx, &pos),
                            file: None,
                            position: None,
                        }
                    })?;

                let ast = SchemaAst::from_model(schema, self.ctx, expanded)?;

                Ok(Some(Node {
                    kind: NodeKind::OpenAPISchema(ast),
                    pos,
                    children: vec![],
                }))
            }
        }
    }

    fn evaluate_option_value(
        &mut self,
        opt: Option<AttributeValue>,
        pos: &Position,
    ) -> Result<Option<Value>> {
        opt.map(|v| match v {
            AttributeValue::Expression(expr) => {
                expressions::parse(&expr).and_then(|ast| self.expr_interpreter.interpret(ast))
            }
            AttributeValue::Literal(s) => Ok(Value::String(s)),
        })
        .map_or(Ok(None), |r| r.map(Some))
        .map_err(|e| Error {
            code: Error::INVALID_EXPRESSION,
            message: "Error in expression".to_string(),
            description: e.render(self.input, self.ctx, None, None, pos),
            file: None,
            position: None,
        })
    }

    fn render_children(&mut self, children: Vec<ContentNode>) -> Result<Vec<Node>> {
        let mut rendered_children = vec![];

        for child in children {
            if let Some(c) = self.walk(child)? {
                rendered_children.push(c)
            }
        }

        Ok(rendered_children)
    }

    fn evaluate_expr(&mut self, expr: &str) -> expressions::Result<Value> {
        let ast = expressions::parse(expr)?;
        self.expr_interpreter.interpret(ast)
    }

    /// Resolves the attributes from the content node to the custom components expected inputs.
    ///
    /// If required attributes are missing, returns an error.
    fn resolve_attributes(
        &mut self,
        component: &CustomComponent,
        attributes: &[Attribute],
        node_pos: &Position,
    ) -> Result<Vec<(String, Value)>> {
        let mut out = vec![];

        let expected_attrs = &component.attributes;

        let incoming_attr_titles: Vec<&String> = attributes.iter().map(|a| &a.key).collect();
        let expected_attr_titles: Vec<&String> = expected_attrs.iter().map(|a| &a.title).collect();

        for incoming in incoming_attr_titles {
            if incoming == "if" || incoming == "elseif" || incoming == "else" {
                // these are special attributes
                continue;
            }

            if !expected_attr_titles.contains(&incoming) {
                // we have unexpected attributes
                return Err(Error {
                    code: Error::INVALID_COMPONENT,
                    message: "Unexpected attribute".to_string(),
                    description: ComponentError::UnexpectedAttribute(
                        incoming.clone(),
                        node_pos.clone(),
                    )
                    .render(self.input, self.ctx),
                    file: None,
                    position: None,
                });
            }
        }

        for attr_spec in expected_attrs {
            if let Some(incoming) = attributes.iter().find(|a| a.key == attr_spec.title) {
                let val = match &incoming.value {
                    Some(AttributeValue::Expression(expr)) => expressions::parse(expr)
                        .and_then(|ast| self.expr_interpreter.interpret(ast))
                        .and_then(|val| attr_spec.verify_incoming(val, expr)),
                    Some(AttributeValue::Literal(s)) => {
                        let val = if let Ok(dec) = Decimal::from_str(s) {
                            Value::Number(dec)
                        } else if s == "true" || s == "false" {
                            Value::Bool(s == "true")
                        } else {
                            Value::String(s.clone())
                        };

                        attr_spec.verify_incoming(val, s)
                    }
                    None => attr_spec.verify_incoming(Value::Bool(true), ""),
                }
                .map_err(|e| Error {
                    code: Error::INVALID_EXPRESSION,
                    message: "Error in attribute expression".to_string(),
                    description: e.render(
                        self.input,
                        self.ctx,
                        Some(&incoming.key),
                        incoming.value.as_ref().map(|v| v.as_str()),
                        node_pos,
                    ),
                    file: None,
                    position: None,
                })?;

                out.push((attr_spec.title.clone(), val));
            } else {
                if attr_spec.required {
                    return Err(Error {
                        code: Error::INVALID_EXPRESSION,
                        message: "Error in attribute expression".to_string(),
                        description: error_renderer::render(
                            self.input,
                            &format!(
                                "Missing required attribute `{}` for component `{}`",
                                attr_spec.title, component.title
                            ),
                            vec![Highlight {
                                span: 1,
                                msg: Some("Missing required attribute".to_string()),
                                location: Location::Point(node_pos.start.row, node_pos.start.col),
                            }],
                            self.ctx,
                        ),
                        file: None,
                        position: None,
                    });
                }

                if let Some(default) = &attr_spec.default {
                    out.push((attr_spec.title.clone(), default.clone().into()));
                } else {
                    out.push((attr_spec.title.clone(), Value::Null));
                }
            }
        }

        Ok(out)
    }

    /// Walk the AST to find any link definitions. Since they can appear at any
    /// point in the document, we can't do the conversion in one pass. We first
    /// need to find all definitions so that we can resolve link references to
    /// their definitions.
    fn find_references(&mut self, node: &ContentNode) {
        if let ContentNodeKind::Definition {
            identifier,
            url,
            title,
            label,
        } = &node.kind
        {
            self.state.definitions.insert(
                identifier.clone(),
                Reference {
                    url: url.clone(),
                    title: title.clone(),
                    label: label.clone(),
                },
            );
        }

        for child in &node.children {
            self.find_references(child)
        }
    }

    fn evaluate_attributes(
        &mut self,
        mut attrs: Vec<Attribute>,
        node_pos: &Position,
    ) -> Result<Vec<Attribute>> {
        let mut out = vec![];

        for mut attr in attrs.drain(..) {
            if let Some(AttributeValue::Expression(ref mut expr)) = attr.value {
                let ast = super::expressions::parse(expr).map_err(|e| Error {
                    code: Error::INVALID_EXPRESSION,
                    message: "Error parsing attribute expression".to_string(),
                    description: e.render(
                        self.input,
                        self.ctx,
                        Some(&attr.key),
                        Some(expr),
                        node_pos,
                    ),
                    file: None,
                    position: None,
                })?;

                let val = self.expr_interpreter.interpret(ast).map_err(|e| Error {
                    code: Error::INVALID_EXPRESSION,
                    message: "Error evaluating attribute expression".to_string(),
                    description: e.render(
                        self.input,
                        self.ctx,
                        Some(&attr.key),
                        Some(expr),
                        node_pos,
                    ),
                    file: None,
                    position: None,
                })?;

                attr.value = Some(AttributeValue::Literal(val.to_string()));
            }

            out.push(attr);
        }

        Ok(out)
    }

    fn is_custom_component(&self) -> bool {
        self.ctx
            .file_context
            .as_ref()
            .map(|f| {
                self.ctx
                    .custom_components
                    .iter()
                    .any(|c| c.path == f.fs_path)
            })
            .unwrap_or(false)
    }
}

#[derive(Debug, Default, PartialEq, Eq, Hash)]
struct Reference {
    pub url: String,
    pub title: Option<String>,
    pub label: Option<String>,
}

#[derive(Debug, Default)]
struct ConversionState {
    pub definitions: HashMap<String, Reference>,
    pub anchorizer: Anchorizer,
}
