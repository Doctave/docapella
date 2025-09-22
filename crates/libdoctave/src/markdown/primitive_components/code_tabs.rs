use crate::{
    autocomplete::PrimitiveComponentAutocomplete,
    expressions::Value,
    markdown::error_renderer::{self, Highlight, Location},
    primitive_components::tabs::TITLE_KEY,
    render_context::RenderContext,
    renderable_ast::Position,
    Node, NodeKind,
};
use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error(r#"Invalid tab"#)]
    InvalidTabNode(Position),
    #[error(r#"Missing a label"#)]
    MissingLabel(Position),
    #[error(r#"Duplicate label"#)]
    DuplicateLabel(Position),
    #[error(r#"Missing {TITLE_KEY}"#)]
    MissingTitle(Position),
    #[error(r#"Missing language"#)]
    MissingLang(Position),
}

impl Error {
    pub(crate) fn render(&self, md: &str, ctx: &RenderContext, _pos: &Position) -> String {
        let mut highlights = vec![];
        match self {
            Error::InvalidTabNode(pos) => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some("Invalid tab node. Expected a code block.".to_string()),
                };

                highlights.push(highlight);
            }
            Error::MissingTitle(pos) => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some(format!("Missing {TITLE_KEY}")),
                };

                highlights.push(highlight);
            }
            Error::MissingLabel(pos) => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some("Add either a language or label for the code block.".to_string()),
                };

                highlights.push(highlight);
            }
            Error::DuplicateLabel(pos) => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some("Labels must be unique".to_string()),
                };

                highlights.push(highlight);
            }
            Error::MissingLang(pos) => {
                let location = Location::Point(pos.start.row, pos.start.col);

                let highlight = Highlight {
                    location,
                    span: 1,
                    msg: Some("Add language to use code attributes".to_string()),
                };

                highlights.push(highlight);
            }
        }

        error_renderer::render(md, &self.to_string(), highlights, ctx)
    }
}

#[derive(Debug, Default, Clone, PartialEq, Deserialize, Serialize)]
pub struct CodeSelect;

#[derive(Default, Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct CodeTab {
    pub label: String,
    pub title: String,
}

impl CodeSelect {
    pub fn verify_code_children(nodes: &Vec<Node>) -> Result<()> {
        let mut found_labels = vec![];

        for node in nodes {
            match &node.kind {
                NodeKind::Code {
                    label,
                    title,
                    language,
                    ..
                } => {
                    if let Some(lang) = language {
                        // special case, it means that language was not supplied and
                        // instead attributes were parsed as the language, e.g.
                        // ``` title=foo
                        if lang.contains('=') {
                            return Err(Error::MissingLang(node.pos.clone()));
                        }
                    }

                    if title.is_none() {
                        return Err(Error::MissingTitle(node.pos.clone()));
                    }

                    let label = label
                        .as_ref()
                        .ok_or(Error::MissingLabel(node.pos.clone()))?;

                    if found_labels.contains(label) {
                        return Err(Error::DuplicateLabel(node.pos.clone()));
                    }

                    found_labels.push(label.to_owned());
                }
                _ => {
                    return Err(Error::InvalidTabNode(node.pos.clone()));
                }
            }
        }

        Ok(())
    }

    pub fn overwrite_code_blocks(nodes: &mut Vec<Node>, parent_title: Option<&Value>) {
        for node in nodes {
            if let NodeKind::Code { title, .. } = &mut node.kind {
                if title.is_none() {
                    *title = parent_title.map(|v| v.to_string());
                }
            }
        }
    }
}

impl PrimitiveComponentAutocomplete for CodeSelect {
    fn title(&self) -> &str {
        "CodeSelect"
    }

    fn attributes(&self) -> Vec<&str> {
        vec!["title"]
    }

    fn attribute_values(&self, _attribute: &str) -> Vec<&str> {
        vec![]
    }
}

#[cfg(test)]
mod test {
    mod code_tabs {
        use pretty_assertions::assert_str_eq;

        use crate::{ast_mdx, render_context::RenderContext};

        #[test]
        fn basic() {
            let input = indoc! {r#"
            <CodeSelect title="foo">
            ```javascript
            const foo = 'bar';
            ```
            ```rust
            let foo = "bar";
            ```
            ```elixir
            foo = "bar"
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let node = &ast_mdx(input, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
              <CodeSelect>
                  <Code language={javascript} title={foo} label={Javascript} raw={false} show_whitespace={false}>
                      const foo = 'bar';
                  </Code>
                  <Code language={rust} title={foo} label={Rust} raw={false} show_whitespace={false}>
                      let foo = "bar";
                  </Code>
                  <Code language={elixir} title={foo} label={Elixir} raw={false} show_whitespace={false}>
                      foo = "bar"
                  </Code>
              </CodeSelect>
              "#}
            );
        }

        #[test]
        fn unexpected_attributes() {
            let markdown = indoc! {r#"
            <CodeSelect foobar="booboo">
            </CodeSelect>
            "#};

            let ctx = RenderContext::new();
            let error = &ast_mdx(markdown, &ctx).unwrap_err();

            assert_str_eq!(error.message, "Unexpected attribute");
            assert_str_eq!(
                error.description,
                indoc! {r#"
                Unexpected attribute "foobar"

                    1 │ <CodeSelect foobar="booboo">
                                    ▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn newlines_in_between() {
            let input = indoc! {r#"
            <CodeSelect title="foo">
            ```javascript
            const foo = 'bar';
            ```

            ```rust
            let foo = "bar";
            ```



            ```elixir
            foo = "bar"
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let node = &ast_mdx(input, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <CodeSelect>
                    <Code language={javascript} title={foo} label={Javascript} raw={false} show_whitespace={false}>
                        const foo = 'bar';
                    </Code>
                    <Code language={rust} title={foo} label={Rust} raw={false} show_whitespace={false}>
                        let foo = "bar";
                    </Code>
                    <Code language={elixir} title={foo} label={Elixir} raw={false} show_whitespace={false}>
                        foo = "bar"
                    </Code>
                </CodeSelect>
                "#}
            );
        }

        #[test]
        fn unwraps_lone_code() {
            let input = indoc! {r#"
            <CodeSelect title="foo">
            ```javascript
            const foo = 'bar';
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let node = &ast_mdx(input, &ctx).unwrap();

            assert_str_eq!(
                node.debug_string().unwrap(),
                indoc! {r#"
                <Code language={javascript} title={foo} label={Javascript} raw={false} show_whitespace={false}>
                    const foo = 'bar';
                </Code>
              "#}
            );
        }

        #[test]
        fn invalid_tab() {
            let input = indoc! {r#"
            <CodeSelect title="foo">
            ```javascript
            const foo = 'bar';
            ```
            ```rust
            let foo = "bar";
            ```

            Foobar!
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let error = &ast_mdx(input, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! {r#"
                Invalid tab

                    8 │
                    9 │ Foobar!
                        ▲
                        └─ Invalid tab node. Expected a code block.

              "#}
            );
        }

        #[test]
        fn missing_label() {
            let input = indoc! {r#"
            <CodeSelect title="foo">
            ```
            const foo = 'bar';
            ```
            ```
            let foo = "bar";
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let error = &ast_mdx(input, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! {r#"
                Missing a label

                    1 │ <CodeSelect title="foo">
                    2 │ ```
                        ▲
                        └─ Add either a language or label for the code block.

                "#}
            );
        }

        #[test]
        fn missing_title() {
            let input = indoc! {r#"
            <CodeSelect>
            ```js
            const foo = 'bar';
            ```
            ```rust
            let foo = "bar";
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let error = &ast_mdx(input, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! {r#"
                Missing title

                    1 │ <CodeSelect>
                    2 │ ```js
                        ▲
                        └─ Missing title

                "#}
            );
        }

        #[test]
        fn missing_language() {
            let input = indoc! {r#"
            <CodeSelect>
            ``` title=foo
            const foo = 'bar';
            ```
            ``` title=bar
            let foo = "bar";
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let error = &ast_mdx(input, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! {r#"
                Missing language

                    1 │ <CodeSelect>
                    2 │ ``` title=foo
                        ▲
                        └─ Add language to use code attributes

                "#}
            );
        }

        #[test]
        fn non_unique_labels() {
            let input = indoc! {r#"
            <CodeSelect title="foo">
            ```js
            const foo = 'bar';
            ```
            ```js
            let foo = "bar";
            ```
            </CodeSelect>
            "#};

            let ctx = RenderContext::default();
            let error = &ast_mdx(input, &ctx).unwrap_err();

            assert_str_eq!(
                error.description,
                indoc! {r#"
                Duplicate label

                    4 │ ```
                    5 │ ```js
                        ▲
                        └─ Labels must be unique

                "#}
            );
        }
    }
}
