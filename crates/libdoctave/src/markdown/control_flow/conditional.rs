use std::{fmt::Display, str::FromStr};

use crate::{
    content_ast::{Node as ContentNode, NodeKind as ContentNodeKind},
    markdown::error_renderer::{self, Highlight, Location},
    render_context::RenderContext,
    renderable_ast::Position,
};
use itertools::{Itertools, PeekingNext};
use thiserror::Error;
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq)]
pub struct Conditional {
    pub op: Operation,
    pub cond_expr: Option<String>,
    pub true_branch: Box<ContentNode>,
    pub false_branch: Box<ContentNode>,
}

impl Conditional {
    /// Verify a whole conditional chain recursively
    pub fn verify(&self, parent_branch: Option<&ContentNode>) -> Result<()> {
        let current_branch = self.true_branch.as_ref();
        let next_branch = self.false_branch.as_ref();

        // Verify that the first branch in the sequence is an if
        if parent_branch.is_none() && self.op != Operation::If {
            return Err(Error::InvalidStart(
                self.clone(),
                current_branch.pos.clone(),
            ));
        }

        // Verify current branch
        match (self.op, &self.cond_expr) {
            (Operation::If, None) | (Operation::ElseIf, None) => {
                return Err(Error::InvalidOpShouldHaveValue(
                    self.clone(),
                    current_branch.pos.clone(),
                ));
            }
            (Operation::Else, Some(_)) => {
                return Err(Error::InvalidOpShouldNotHaveValue(
                    self.clone(),
                    current_branch.pos.clone(),
                ));
            }
            _ => {}
        }

        // Verify the chain to next branch if there is one
        if let ContentNodeKind::Conditional(next_cond) = &next_branch.kind {
            match (self.op, next_cond.op) {
                (_, Operation::If) => {
                    panic!("This is a bug. We didn't detect the start of a new sequence during zipping.")
                }
                (Operation::Else, _) => {
                    return Err(Error::InvalidChain(
                        self.clone(),
                        next_cond.clone(),
                        next_branch.pos.clone(),
                    ));
                }
                _ => {}
            }

            next_cond.verify(Some(current_branch))?;
        }

        Ok(())
    }

    /// Push to the end of conditional chain
    pub fn push(&mut self, new_branch: ContentNode) -> Result<()> {
        match &mut self.false_branch.kind {
            ContentNodeKind::Noop => {
                self.false_branch = Box::new(new_branch.clone());

                Ok(())
            }
            ContentNodeKind::Conditional(cond) => cond.push(new_branch),
            _ => Ok(()),
        }
    }
}

/// Folds any conditional sequences in a list of nodes together.
/// Also clears line breaks between conditional nodes in a conditional sequence (due to MDX parser).
///
/// Example:
/// ```ignore
/// vec![                     vec![
///   cond: if                  cond: if
///   cond: elseif                cond elseif
///   cond: else    ------>         cond: else
///   node: text                node: text
///   cond: if                  cond: if
///   cond: if                  cond: if
/// ]
/// ```
pub fn fold_conditionals(children: Vec<ContentNode>) -> Result<Vec<ContentNode>> {
    let mut new_children = vec![];
    let mut iter = children.into_iter().multipeek();

    while let Some(mut current_node) = iter.next() {
        let mut in_between: Vec<ContentNode> = vec![];
        if let ContentNodeKind::Conditional(cond) = &mut current_node.kind {
            // Go through siblings until we find the start of a new chain ("if")
            // Push any conditionals to the current chain and consume other elements in between
            while let Some(real_next) =
                iter.peeking_next(|fake_next| !fake_next.is_start_of_sequence())
            {
                if real_next.is_conditional() {
                    cond.push(real_next)?;

                    // Verify what is flushed out in between nodes
                    for node in &in_between {
                        if !node.is_whitespace_or_linebreak() {
                            return Err(Error::InvalidInBetweenChain(
                                node.clone(),
                                node.pos.clone(),
                            ));
                        }
                    }

                    in_between = vec![];
                } else {
                    in_between.push(real_next);
                }
            }

            cond.verify(None)?;
        }

        new_children.push(current_node);
        new_children.append(&mut in_between);
    }

    Ok(new_children)
}

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Conditional should start with an "if""#)]
    InvalidStart(Conditional, Position),
    #[error(r#"Conditional "{}" can't be followed by an "{}""#, .0.op, .1.op)]
    InvalidChain(Conditional, Conditional, Position),
    #[error(r#"Conditional "{}" must have a value"#, .0.op)]
    InvalidOpShouldHaveValue(Conditional, Position),
    #[error(r#"Conditional "{}" must not have a value"#, .0.op)]
    InvalidOpShouldNotHaveValue(Conditional, Position),
    #[error(r#"Conditionals can't have anything in between"#)]
    InvalidInBetweenChain(ContentNode, Position),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext) -> String {
        let op_pos = |node: &ContentNode, op: &Operation, offset: Option<usize>| {
            let offset = offset.unwrap_or(0);
            let node_string = md
                .chars()
                .skip(node.pos.start.byte_offset)
                .take(node.pos.end.byte_offset)
                .collect::<String>();

            if let Some(start_pos) = node_string.find(op.as_str()) {
                Location::Point(node.pos.start.row, node.pos.start.col + start_pos + offset)
            } else {
                panic!("This is a bug. The conditional branch didn't find its own operation from itself.")
            }
        };
        let highlights = match self {
            Error::InvalidChain(current_cond, new_cond, _) => {
                let current_node = current_cond.true_branch.as_ref();
                let current_op = &current_cond.op;

                let new_node = new_cond.true_branch.as_ref();
                let new_op = &new_cond.op;

                let current_op_location = op_pos(current_node, current_op, None);
                let new_op_location = op_pos(new_node, new_op, None);

                let current_highlight = Highlight {
                    span: 1,
                    location: current_op_location,
                    msg: Some(format!(r#"This "{current_op}""#)),
                };

                let new_highlight = Highlight {
                    span: 1,
                    location: new_op_location,
                    msg: Some(format!(r#"can't be followed by "{new_op}""#)),
                };

                vec![current_highlight, new_highlight]
            }
            Error::InvalidStart(
                Conditional {
                    op, true_branch, ..
                },
                _,
            ) => {
                let op_location = op_pos(true_branch.as_ref(), op, None);

                let highlight = Highlight {
                    span: 1,
                    location: op_location,
                    msg: Some(format!(r#"Replace "{op}" with "if""#)),
                };
                vec![highlight]
            }
            Error::InvalidOpShouldHaveValue(
                Conditional {
                    op, true_branch, ..
                },
                _,
            ) => {
                let op_location = op_pos(true_branch.as_ref(), op, None);

                let highlight = Highlight {
                    span: 1,
                    location: op_location,
                    msg: Some(format!(r#"{op}={{"some_value_here"}}"#)),
                };
                vec![highlight]
            }
            Error::InvalidOpShouldNotHaveValue(
                Conditional {
                    op, true_branch, ..
                },
                _,
            ) => {
                // offset the length of op + "={"
                let op_location = op_pos(true_branch.as_ref(), op, Some(op.as_str().len() + 2));

                let highlight = Highlight {
                    span: 1,
                    location: op_location,
                    msg: Some(format!(r#"Remove the value for "{op}""#)),
                };
                vec![highlight]
            }
            Error::InvalidInBetweenChain(
                ContentNode {
                    pos: Position { start, .. },
                    kind,
                    ..
                },
                _,
            ) => {
                // if there is multi-line text, we must adjust position by the newlines at start
                // TODO: improve multi line erroring
                let highlight = if let ContentNodeKind::Text { value } = kind {
                    let row_indent = value.chars().take_while(|c| c == &'\n').count();

                    if row_indent > 0 {
                        Highlight {
                            span: 1,
                            location: Location::Point(start.row + row_indent, 1),
                            msg: Some("Remove this row".to_string()),
                        }
                    } else {
                        Highlight {
                            span: value.len(),
                            location: Location::Point(start.row, start.col),
                            msg: Some("Remove this value".to_string()),
                        }
                    }
                } else {
                    Highlight {
                        span: 1,
                        location: Location::Point(start.row, start.col),
                        msg: Some("Remove this value".to_string()),
                    }
                };

                vec![highlight]
            }
        };

        error_renderer::render(md, &format!("{}", self), highlights, ctx)
    }

    pub fn position(&self) -> Position {
        match self {
            Error::InvalidStart(_, pos) => pos.clone(),
            Error::InvalidChain(_, _, pos) => pos.clone(),
            Error::InvalidOpShouldHaveValue(_, pos) => pos.clone(),
            Error::InvalidOpShouldNotHaveValue(_, pos) => pos.clone(),
            Error::InvalidInBetweenChain(_, pos) => pos.clone(),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum Operation {
    If,
    ElseIf,
    Else,
}

impl Display for Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Operation {
    type Err = String;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        use Operation::*;

        match s {
            "if" => Ok(If),
            "elseif" => Ok(ElseIf),
            "else" => Ok(Else),
            _ => Err("could not parse operation".to_string()),
        }
    }
}

impl Operation {
    pub fn as_str(&self) -> &str {
        use Operation::*;

        match self {
            If => "if",
            ElseIf => "elseif",
            Else => "else",
        }
    }
}

#[cfg(test)]
mod test {
    use crate::ast_mdx;
    use crate::render_context::RenderContext;
    use pretty_assertions::assert_str_eq;

    #[test]
    fn conditionally_renders_the_children() {
        let markdown = indoc! {"
        <Fragment if={1 > 2}>The text</Fragment>
        "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(root.debug_string().unwrap(), indoc! {""});
    }

    #[test]
    fn conditionally_renders_the_children_if() {
        let markdown = indoc! {"
      <Fragment if={3 > 2}>The text</Fragment>
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
    fn conditionally_renders_the_children_if_with_some_element_after() {
        let markdown = indoc! {"
        <Fragment if={3 > 2}>The text</Fragment>
        Hello world!
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
                <Text>

                    Hello world!
                </Text>
            </Paragraph>
            "}
        );
    }

    #[test]
    fn conditionally_renders_the_children_elseif() {
        let markdown = indoc! {"
        <Fragment if={1 > 2}>The text</Fragment>
        <Fragment elseif={3 > 2}>Elseif text</Fragment>
        "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
            <Paragraph>
                <Text>
                    Elseif text
                </Text>
            </Paragraph>
            "}
        );
    }

    #[test]
    fn conditionally_renders_the_children_multiple_elseif() {
        let markdown = indoc! {"
        <Fragment if={1 > 2}>The text</Fragment>
        <Fragment elseif={1 > 2}>Elseif text 1</Fragment>
        <Fragment elseif={3 > 2}>Elseif text 2</Fragment>
        "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
            <Paragraph>
                <Text>
                    Elseif text 2
                </Text>
            </Paragraph>
            "}
        );
    }

    #[test]
    fn conditionally_renders_the_children_else() {
        let markdown = indoc! {"
        <Fragment if={1 > 2}>The text</Fragment>
        <Fragment elseif={1 > 2}>Elseif text</Fragment>
        <Fragment else>Else text</Fragment>
        "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
            <Paragraph>
                <Text>
                    Else text
                </Text>
            </Paragraph>
            "}
        );
    }

    #[test]
    fn conditionally_renders_the_children_nested() {
        let markdown = indoc! {"
        <Fragment if={true}>
            <Fragment if={false}>Nested text</Fragment>
            <Fragment else>Nested else text</Fragment>
        </Fragment>
        "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
            <Paragraph>
                <Text>
                    Nested else text
                </Text>
            </Paragraph>
            "}
        );
    }

    #[test]
    fn conditionally_renders_two_ifs_in_row_keeps_line_breaks() {
        let markdown = indoc! {"
        <Fragment if={true}>Text 1</Fragment>
        <Fragment if={true}>Text 2</Fragment>
        "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
            <Paragraph>
                <Text>
                    Text 1
                </Text>
                <Text>


                </Text>
                <Text>
                    Text 2
                </Text>
            </Paragraph>
            "}
        );
    }

    #[test]
    fn conditionally_renders_the_children_else_orphan() {
        let markdown = indoc! {"
        <Fragment else>The text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! { r#"
          Conditional should start with an "if"

              1 │ <Fragment else>The text</Fragment>
                            ▲
                            └─ Replace "else" with "if"

          "# }
        );
    }

    #[test]
    fn conditionally_renders_the_children_else_orphan_not_inline() {
        let markdown = indoc! {"
        <Fragment else>
            The text
        </Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! { r#"
          Conditional should start with an "if"

              1 │ <Fragment else>
                            ▲
                            └─ Replace "else" with "if"

          "# }
        );
    }

    #[test]
    fn conditionally_renders_the_children_elseif_orphan() {
        let markdown = indoc! {"
        <Fragment elseif={1 > 2}>The text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! { r#"
          Conditional should start with an "if"

              1 │ <Fragment elseif={1 > 2}>The text</Fragment>
                            ▲
                            └─ Replace "elseif" with "if"

          "#}
        );
    }

    #[test]
    fn conditionally_renders_the_children_messed_up() {
        let markdown = indoc! {"
        <Fragment if={3 > 2}>
            Elseif text
        </Fragment>
        <Fragment elseif={3 > 2}>If text</Fragment>

        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! {r#"
          Conditional should start with an "if"

              3 │ </Fragment>
              4 │ <Fragment elseif={3 > 2}>If text</Fragment>
                            ▲
                            └─ Replace "elseif" with "if"

          "#}
        );
    }

    #[test]
    fn conditionally_renders_the_children_messed_up_2() {
        let markdown = indoc! {"
        <Fragment if={3 > 2}>If text</Fragment>
        <Fragment else>Else text</Fragment>
        <Fragment elseif={1 > 2}>Elseif text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! {
              r#"
        Conditional "else" can't be followed by an "elseif"

            1 │ <Fragment if={3 > 2}>If text</Fragment>
            2 │ <Fragment else>Else text</Fragment>
                          ▲
                          └─ This "else"
            3 │ <Fragment elseif={1 > 2}>Elseif text</Fragment>
                          ▲
                          └─ can't be followed by "elseif"

        "#
            }
        );
    }

    #[test]
    fn conditionally_if_doesnt_have_value() {
        let markdown = indoc! {"
        <Fragment if>If text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! {r#"
          Conditional "if" must have a value

              1 │ <Fragment if>If text</Fragment>
                            ▲
                            └─ if={"some_value_here"}

          "#}
        );
    }

    #[test]
    fn conditionally_elseif_doesnt_have_value() {
        let markdown = indoc! {"
        <Fragment elseif>If text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! {r#"
          Conditional should start with an "if"

              1 │ <Fragment elseif>If text</Fragment>
                            ▲
                            └─ Replace "elseif" with "if"

          "#}
        );
    }

    #[test]
    fn conditionally_elseif_doesnt_have_value_in_a_sequence() {
        let markdown = indoc! {"
        <Fragment if={false}>If text</Fragment>
        <Fragment elseif>If text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! {r#"
            Conditional "elseif" must have a value

                1 │ <Fragment if={false}>If text</Fragment>
                2 │ <Fragment elseif>If text</Fragment>
                              ▲
                              └─ elseif={"some_value_here"}

          "#}
        );
    }

    #[test]
    fn conditionally_else_has_value() {
        let markdown = indoc! {"
        <Fragment if={true}>If text</Fragment>
        <Fragment else={false}>If text</Fragment>
        "};

        let ctx = RenderContext::new();
        let err = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            err.description,
            indoc! {r#"
          Conditional "else" must not have a value

              1 │ <Fragment if={true}>If text</Fragment>
              2 │ <Fragment else={false}>If text</Fragment>
                                  ▲
                                  └─ Remove the value for "else"

          "#}
        );
    }

    #[test]
    fn conditionally_errors_from_nodes_in_between() {
        let markdown = indoc! {"
          <Fragment if={1 > 2}>The text</Fragment>
          hello
          in between
          <Fragment elseif={3 > 2}>Elseif text</Fragment>
          world
          <Fragment else>Else text</Fragment>
          "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {"
            Conditionals can't have anything in between

                1 │ <Fragment if={1 > 2}>The text</Fragment>
                2 │ hello
                    ▲
                    └─ Remove this row

            "}
        );
    }

    #[test]
    fn conditionally_errors_from_nodes_in_between_inline() {
        let markdown = indoc! {"
          <Fragment if={1 > 2}>The text</Fragment>hello in between<Fragment else>Else text</Fragment>
          "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {"
            Conditionals can't have anything in between

                1 │ <Fragment if={1 > 2}>The text</Fragment>hello in between<Fragment else>Else text</Fragment>
                                                            ▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲▲
                                                            └─ Remove this value

            "}
        );
    }

    #[test]
    fn conditionally_errors_from_nodes_in_between_breaks_chain() {
        let markdown = indoc! {"
          <Fragment if={1 > 2}>The text</Fragment>
          <Box>
            hello
            in between
          </Box>
          <Fragment elseif={3 > 2}>Elseif text</Fragment>
          <Fragment else>Else text</Fragment>
          "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Conditional should start with an "if"

                5 │ </Box>
                6 │ <Fragment elseif={3 > 2}>Elseif text</Fragment>
                              ▲
                              └─ Replace "elseif" with "if"

            "#}
        );
    }

    #[test]
    fn conditionally_errors_from_nodes_in_between_breaks_chain_with_in_between() {
        let markdown = indoc! {"
          <Fragment if={1 > 2}>The text</Fragment>
          <Box>
            hello
            in between
          </Box>
          <Fragment elseif={3 > 2}>Elseif text</Fragment>
          hello
          <Fragment else>Else text</Fragment>
          "};

        let ctx = RenderContext::new();
        let error = ast_mdx(markdown, &ctx).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! {r#"
            Conditionals can't have anything in between

                6 │ <Fragment elseif={3 > 2}>Elseif text</Fragment>
                7 │ hello
                    ▲
                    └─ Remove this row

            "#}
        );
    }

    #[test]
    fn conditionally_keeps_any_siblings_in_between_starts() {
        let markdown = indoc! {"
          <Fragment if={1 > 2}>The text</Fragment>
          hello in between
          <Fragment if={2>1}>Else text</Fragment>
          "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
              <Paragraph>
                  <Text>

                      hello in between

                  </Text>
                  <Text>
                      Else text
                  </Text>
              </Paragraph>
              "}
        );
    }

    #[test]
    fn conditionally_keeps_any_siblings_in_between_starts_inline() {
        let markdown = indoc! {"
          <Fragment if={1 > 2}>The text</Fragment>hello in between<Fragment if={2>1}>Else text</Fragment>
          "};

        let ctx = RenderContext::new();
        let root = ast_mdx(markdown, &ctx).unwrap();

        assert_str_eq!(
            root.debug_string().unwrap(),
            indoc! {"
              <Paragraph>
                  <Text>
                      hello in between
                  </Text>
                  <Text>
                      Else text
                  </Text>
              </Paragraph>
              "}
        );
    }
}
