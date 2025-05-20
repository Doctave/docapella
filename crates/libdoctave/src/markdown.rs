use crate::render_context::RenderContext;
use crate::{Project, Result};

mod anchorizer;
pub mod attribute_parser;
mod components;
pub(crate) mod content_ast;
pub mod control_flow;
pub(crate) mod error_renderer;
pub mod expressions;
pub(crate) mod interpreter;
mod markdown_rs_error_wrapper;
pub mod parser;
pub mod primitive_components;
pub mod renderable_ast;
pub(crate) mod shared_ast;

pub mod autocomplete;
mod custom_components;
mod sanitizer;

pub use anchorizer::Anchorizer;
pub(crate) use custom_components::custom_component::{
    CustomComponent, CustomComponentHandle, BAKED_COMPONENTS,
};
pub use renderable_ast::{Attribute, AttributeValue, Node, NodeKind, TableAlignment};

use std::path::Path;

pub(crate) fn ast(markdown: &str, ctx: &RenderContext) -> Result<Node> {
    parser::to_ast(markdown, ctx)
}

pub(crate) fn ast_mdx(markdown: &str, ctx: &RenderContext) -> Result<Node> {
    parser::to_ast_mdx(markdown, ctx)
}

pub(crate) fn autocomplete(
    markdown: &str,
    fs_path: &Path,
    project: &Project,
    ctx: &RenderContext,
) -> Vec<autocomplete::CompletionItem> {
    autocomplete::autocomplete(markdown, fs_path, project, ctx)
}

#[allow(dead_code)]
pub(crate) fn ast_mdx_fault_tolerant(
    markdown: &str,
    ctx: &RenderContext,
) -> std::result::Result<Node, (Option<Node>, Vec<crate::Error>)> {
    parser::to_ast_mdx_fault_tolerant(markdown, ctx)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::RenderOptions;
    use indoc::indoc;
    use pretty_assertions::assert_str_eq;
    use std::collections::HashMap;

    mod v2_templates {
        use std::path::PathBuf;

        use crate::render_context::FileContext;
        use pretty_assertions::assert_eq;

        use super::*;

        #[test]
        fn can_escape_expressions() {
            let markdown = indoc! {r#"
            Curly braces: \{ }
            "#};

            let ctx = RenderContext::new();
            let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

            assert_str_eq!(
                html,
                indoc! { r#"
                <Paragraph>
                    <Text>
                        Curly braces: { }
                    </Text>
                </Paragraph>
                "# }
            );
        }

        #[test]
        fn executes_expressions() {
            let markdown = indoc! {r#"
            1 + 1 is { 1 + 1 }
            "#};

            let ctx = RenderContext::new();
            let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

            assert_str_eq!(
                html,
                indoc! { r#"
                <Paragraph>
                    <Text>
                        1 + 1 is
                    </Text>
                    <Text>
                        2
                    </Text>
                </Paragraph>
                "# }
            );
            // "<p>1 + 1 is 2</p>\n"
        }

        #[test]
        fn executes_block_expressions() {
            let markdown = indoc! {r#"
            1 + 1 is

            { 1 + 1 }
            "#};

            let ctx = RenderContext::new();
            let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

            assert_str_eq!(
                html,
                indoc! { r#"
                <Paragraph>
                    <Text>
                        1 + 1 is
                    </Text>
                </Paragraph>
                <Paragraph>
                    <Text>
                        2
                    </Text>
                </Paragraph>
                "# }
            );
        }

        #[test]
        fn evaluates_user_preferences() {
            let markdown = indoc! {r#"
            { @user_preferences.plan }
            "#};

            let opts = RenderOptions {
                user_preferences: HashMap::from([("plan".to_string(), "enterprise".to_string())]),
                ..Default::default()
            };

            let mut ctx = RenderContext::new();
            ctx.with_options(&opts);

            let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

            assert_str_eq!(
                html,
                indoc! { r#"
                <Paragraph>
                    <Text>
                        enterprise
                    </Text>
                </Paragraph>
                "# }
            );
        }

        #[test]
        fn supports_tables() {
            let markdown = indoc! {r#"
            |foo|
            |---|
            |bar|
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            let table = root.children.first().unwrap();
            assert!(matches!(table.kind, NodeKind::Table { .. }));
        }

        #[test]
        fn supports_task_lists() {
            let markdown = indoc! {r#"
            - [x] foo
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            let list = root.children.first().unwrap();
            assert!(matches!(list.kind, NodeKind::List { .. }));

            let item = list.children.first().unwrap();
            assert!(matches!(
                item.kind,
                NodeKind::ListItem {
                    checked: Some(true),
                    ..
                }
            ));
        }

        #[test]
        fn conditionally_hides_things() {
            let markdown = indoc! {r#"
            <Button if={false}>
                Hello
            </Button>

            <Button if={false}>Hello</Button>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_eq!(root.children.len(), 0);
        }

        #[test]
        fn conditionally_renders_things() {
            let markdown = indoc! {r#"
            <Button href="https://example.com" if={true}>
                Hello
            </Button>

            <Button href="https://example.com" if={true}>Hello</Button>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_eq!(root.children.len(), 2);
        }

        #[test]
        fn handles_liquid_inputs_gracefully() {
            let markdown = indoc! {r#"
            Some liquid:

            {% capture foo %}
            Capture me!
            {% endcapture %}
            "#};

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert!(
                err.description.contains("Unexpected token `%`"),
                "Incorrect error: {}",
                err.description
            );
        }

        #[test]
        fn evaluates_component_attribute_expressions() {
            let markdown = indoc! {r#"
            <Box pad={2 + 2}>
                Bar
            </Box>
            "#}
            .trim();

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <Box padding={4} max_width={full} class={} height={Auto}>
                    <Paragraph>
                        <Text>
                            Bar
                        </Text>
                    </Paragraph>
                </Box>
              "# }
            )
        }

        #[test]
        fn doc_1136_unwraps_paragraphs() {
            // https://linear.app/doctave/issue/DOC-1136/extra-paragraph-tags-inserted-with-using-a-raw-p-tag
            // https://github.com/wooorm/markdown-rs/issues/116
            // https://github.com/wooorm/mdxjs-rs/blob/b1971be2dbdd2886ca4d5e9d97d9a3477cb29904/src/mdast_util_to_hast.rs#L925
            let markdown = indoc! {r#"
            <p>Foo</p>
            "#};

            let ctx = RenderContext::new();
            let root = ast_mdx(markdown, &ctx).unwrap();

            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <p>
                    <Text>
                        Foo
                    </Text>
                </p>
              "# }
            )
        }

        #[test]
        fn returns_errors_from_component_attr_expressions() {
            let markdown = indoc! {r#"
            <Card pad={2 + true}>
                Bar
            </Card>
            "#}
            .trim();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                err.description,
                indoc! { r#"
                Cannot apply operation `+` on values `2` with type `number` and `true` with type `bool`

                    1 │ <Card pad={2 + true}>
                                   ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn returns_syntax_errors_from_component_attr_expressions() {
            let markdown = indoc! {r#"
            <Card pad={2 + }>
                Bar
            </Card>
            "#}
            .trim();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                err.description,
                indoc! { r#"
                Unexpected end of expression

                    1 │ <Card pad={2 + }>
                                      ▲

                "#}
            );
        }

        #[test]
        fn gives_nice_error_messages_for_broken_mdx_syntax() {
            let markdown = indoc! {r#"
            <Foo>
                Bar
            <Fo
            "#}
            .trim();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Expected a closing tag for `<Foo>`

                    1 │ <Foo>
                        ▲
                        └─ Opening tag
                    2 │     Bar
                    3 │ <Fo
                           ▲
                           └─ Expected close tag

                "#}
            );
        }

        #[test]
        fn gives_nice_error_messages_for_expression_errors() {
            let markdown = indoc! {r#"
            Something

            { (1 + (true + 2)) }

            Else
            "#};

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Cannot apply operation `+` on values `true` with type `bool` and `2` with type `number`

                    2 │
                    3 │ { (1 + (true + 2)) }
                                ▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn can_offset_the_error_message_lines() {
            let markdown = indoc! {r#"
            Something

            { (1 + ("foo" + 2)) }

            Else
            "#};

            let mut ctx = RenderContext::new();
            ctx.with_file_context(FileContext::new(3, 0, PathBuf::from("foo.md")));

            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Cannot apply operation `+` on values `"foo"` with type `string` and `2` with type `number`

                    5 │
                    6 │ { (1 + ("foo" + 2)) }
                                ▲▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        #[ignore]
        // TODO(Nik): This was complicated. The data is there, writing the code is just
        // finicky. Come back to this later when you're unifying the error reporting
        fn can_handle_expressions_on_multiple_lines() {
            let markdown = indoc! {r#"
            Something

            {
              "foo" + 2
            }

            Else
            "#};

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Cannot apply operation `+` on values `"foo"` with type `string` and `2` with type `number`

                    1 │ Something
                    2 │
                    3 │ {
                    4 │   "foo" + 2
                          ▲▲▲▲▲▲▲▲▲
                    5 │ }
                    6 │
                    7 │ Else

                "#}
            );
        }

        #[test]
        fn can_offset_the_error_message_by_the_tags_starting_point() {
            let markdown = indoc! {r#"
            Something

            Calculate: { (1 + ("foo" + 2)) }

            Else
            "#};

            let mut ctx = RenderContext::new();
            ctx.with_file_context(FileContext::new(3, 0, PathBuf::from("foo.md")));

            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Cannot apply operation `+` on values `"foo"` with type `string` and `2` with type `number`

                    5 │
                    6 │ Calculate: { (1 + ("foo" + 2)) }
                                           ▲▲▲▲▲▲▲▲▲

                "#}
            );
        }

        #[test]
        fn nice_error_about_broken_close_tags() {
            let markdown = indoc! {r#"
            Context

            <Foo>
                Bar
            </D>

            More context
            "#}
            .trim_start();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected closing tag `</D>`, expected corresponding closing tag for `<Foo>`

                    2 │
                    3 │ <Foo>
                        ▲
                        └─ Opening tag
                    4 │     Bar
                    5 │ </D>
                        ▲
                        └─ Expected close tag

                "#}
            );
        }

        #[test]
        fn nice_error_about_close_tag_with_attribute() {
            let markdown = "<b> c </b d>";

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected attribute in closing tag, expected the end of the tag

                    1 │ <b> c </b d>
                                  ▲
                                  └─ Unexpected attribute

                "#}
            );
        }

        #[test]
        fn nice_error_about_broken_close_tags_on_one_line() {
            let markdown = indoc! {r#"
            <Foo> bar </D>
            "#}
            .trim_start();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected closing tag `</D>`, expected corresponding closing tag for `<Foo>`

                    1 │ <Foo> bar </D>
                        ▲         ▲
                        │         ╵
                        └─ Opening tag
                                  ╷
                                  └─ Expected close tag

                "#}
            );
        }

        #[test]
        fn nice_error_about_unexpected_closing_tag() {
            let markdown = indoc! {r#"a </B>"#}.trim_start();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected closing slash `/` in tag, expected an open tag first

                    1 │ a </B>
                           ▲
                           └─ Closing slash in tag

                "#}
            );
        }

        #[test]
        fn nice_error_about_interleaving_md_and_mdx_tags() {
            let markdown = indoc! {r#"<a>b *c</a> d*."#}.trim_start();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Expected the closing tag `</a>` either before the start of `Emphasis`, or another opening tag after that start

                    1 │ <a>b *c</a> d*.
                             ▲ ▲
                             │ ╵
                             └─ Opened tag
                               ╷
                               └─ Closing tag

                "#}
            );
        }

        #[test]
        fn nice_error_about_unexpected_self_closing_tag() {
            let markdown = indoc! {r#"a <B> c </B/> d"#}.trim_start();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected self-closing slash `/` in closing tag, expected the end of the tag

                    1 │ a <B> c </B/> d
                                   ▲
                                   └─ Unexpected self-closing slash

                "#}
            );
        }

        #[test]
        fn nice_error_about_html_comments() {
            let markdown = indoc! {r#"* <!a>\n1. b"#}.trim_start();

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected character `!` (U+0021) before name, expected a character that can start a name, such as a letter, `$`, or `_`

                    1 │ * <!a>\n1. b
                           ▲
                           └─ Unexpected character

                "#}
            );
        }

        #[test]
        fn nice_error_about_lazy_flow() {
            let markdown = "> <X\n/>";

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected start of line for component inside container.
                Expected each line of the container be prefixed with `>` when inside a block quote, whitespace when inside a list, etc.

                    1 │ > <X
                    2 │ />
                        ▲
                        └─ Unexpected start of line

                "#}
            );
        }

        #[test]
        fn nice_error_about_lazy_flow_in_expressions() {
            let markdown = "> <a b={c\nd}/>";

            let ctx = RenderContext::new();
            let err = ast_mdx(markdown, &ctx).unwrap_err();

            assert_eq!(
                &err.description,
                indoc! {r#"
                Unexpected start of line in expression inside container.
                Expected each line of the container be prefixed with `>` when inside a block quote, whitespace when inside a list, etc.

                    1 │ > <a b={c
                    2 │ d}/>
                        ▲
                        └─ Unexpected start of line

                "#}
            );
        }

        mod custom_components {
            use super::*;

            #[test]
            fn renders_attributes() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: title
                    required: true
                ---

                ## { @title }
                "#};

                let markdown = "<Component.Example title=\"Example title\" />";

                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_components/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let root = ast_mdx(markdown, &ctx).unwrap();

                let component_root = &root.children[0];
                assert!(matches!(component_root.kind, NodeKind::Root));

                let heading = &component_root.children[0];
                assert!(matches!(heading.kind, NodeKind::Heading { level: 2, .. }));

                let text = &heading.children[0];

                if let NodeKind::Text { value } = &text.kind {
                    pretty_assertions::assert_eq!(value, "Example title");
                } else {
                    panic!("Invalid node in header: {:#?}", text);
                }
            }

            #[test]
            fn renders_a_null_if_attribute_not_passed_in_to_component() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: title
                ---

                ## { @title }
                "#};

                let markdown = "<Component.Example />";

                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_components/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let root = ast_mdx(markdown, &ctx).unwrap();

                let component_root = &root.children[0];
                assert!(matches!(component_root.kind, NodeKind::Root));

                let heading = &component_root.children[0];
                assert!(matches!(heading.kind, NodeKind::Heading { level: 2, .. }));

                let text = &heading.children[0];

                if let NodeKind::Text { value } = &text.kind {
                    pretty_assertions::assert_eq!(value, "");
                } else {
                    panic!("Invalid node in header: {:#?}", text);
                }
            }

            #[test]
            fn passes_on_variables_into_custom_component_that_can_be_used_in_component_attrs() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: pad
                ---

                <Box pad={@pad}>
                   Hi
                </Box>
                "#};

                let markdown = "<Component.Example pad=\"5\" />";

                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_components/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let root = ast_mdx(markdown, &ctx).unwrap();

                assert_str_eq!(
                    root.debug_string().unwrap(),
                    indoc! { r#"
                    <Box padding={5} max_width={full} class={} height={Auto}>
                        <Paragraph>
                            <Text>
                                Hi
                            </Text>
                        </Paragraph>
                    </Box>
                  "# }
                );
            }

            #[test]
            fn topics_passes_on_variables_into_custom_topic_that_can_be_used_in_topic_attrs() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: heading
                    required: true
                ---

                <Box pad="3" class="custom-card-class">
                  **{ @heading }**

                  <Slot />
                </Box>
                "#};

                let markdown = "<Topic.Example heading=\"Hello\">from topics!</Topic.Example>";

                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_topics/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let root = ast_mdx(markdown, &ctx).unwrap();

                assert_str_eq!(
                    root.debug_string().unwrap(),
                    indoc! { r#"
                    <Box padding={3} max_width={full} class={custom-card-class} height={Auto}>
                        <Paragraph>
                            <Strong>
                                <Text>
                                    Hello
                                </Text>
                            </Strong>
                        </Paragraph>
                        <Text>
                            from topics!
                        </Text>
                    </Box>
                    "# }
                );
            }

            #[test]
            fn renders_slot() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: title
                    required: true
                ---

                ## { @title }

                <Slot />
                "#};

                let markdown = indoc! {r#"
                <Component.Example title="Example title">
                    Hello, world
                </Component.Example>
                "#};

                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_components/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let root = ast_mdx(markdown, &ctx).unwrap();

                let component_root = &root.children[0];
                assert!(matches!(component_root.kind, NodeKind::Root));

                let heading = &component_root.children[0];
                assert!(matches!(heading.kind, NodeKind::Heading { level: 2, .. }));

                let slot_root = &component_root.children[1];
                assert!(matches!(slot_root.kind, NodeKind::Root));

                let paragraph = &slot_root.children[0];
                assert!(matches!(paragraph.kind, NodeKind::Paragraph));

                let text = &paragraph.children[0];

                if let NodeKind::Text { value } = &text.kind {
                    pretty_assertions::assert_eq!(value, "Hello, world");
                } else {
                    panic!("Invalid node in header: {:#?}", text);
                }
            }

            #[test]
            fn topics_renders_slot() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: title
                    required: true
                ---

                ## { @title }

                <Slot />
                "#};

                let markdown = indoc! {r#"
                <Topic.Example title="Example title">
                    Hello, world
                </Topic.Example>
                "#};

                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_topics/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let root = ast_mdx(markdown, &ctx).unwrap();

                let component_root = &root.children[0];
                assert!(matches!(component_root.kind, NodeKind::Root));

                let heading = &component_root.children[0];
                assert!(matches!(heading.kind, NodeKind::Heading { level: 2, .. }));

                let slot_root = &component_root.children[1];
                assert!(matches!(slot_root.kind, NodeKind::Root));

                let paragraph = &slot_root.children[0];
                assert!(matches!(paragraph.kind, NodeKind::Paragraph));

                let text = &paragraph.children[0];

                if let NodeKind::Text { value } = &text.kind {
                    pretty_assertions::assert_eq!(value, "Hello, world");
                } else {
                    panic!("Invalid node in header: {:#?}", text);
                }
            }

            #[test]
            fn validates_required_attributes() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: title
                    required: true
                ---

                ## { @title }
                "#};

                let markdown = "<Component.Example />";
                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_components/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let err = ast_mdx(markdown, &ctx).unwrap_err();

                pretty_assertions::assert_eq!(
                    err.description,
                    indoc! {r#"
                    Missing required attribute `title` for component `Component.Example`

                        1 │ <Component.Example />
                            ▲
                            └─ Missing required attribute

                "#}
                );
            }

            #[test]
            fn renders_error_if_expr_error_in_component() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: title
                    required: true
                ---

                ## { @i_dont_exist }
                "#};

                let markdown = "<Component.Example title=\"foo\" />";
                let components = vec![CustomComponentHandle::new(
                    custom_component_template,
                    "_components/example",
                )];

                let mut ctx = RenderContext::new();
                ctx.custom_components = components.as_slice();

                let err = ast_mdx(markdown, &ctx).unwrap_err();

                pretty_assertions::assert_eq!(
                    err.description,
                    indoc! {r#"
                    Variable `@i_dont_exist` not found

                        6 │
                        7 │ ## { @i_dont_exist }
                                 ▲▲▲▲▲▲▲▲▲▲▲▲▲

                "#}
                );
            }

            #[test]
            fn interprets_attributes_without_values_as_true_booleans() {
                let custom_component_template = indoc! {r#"
                ---
                attributes:
                  - title: button
                    required: true
                ---

                <Button href="https://example.com" if={@button}>
                  Hi
                </Button>

                Hi
                "#};

                let markdown = "<Component.Example button />";
                let component =
                    CustomComponentHandle::new(custom_component_template, "_components/example");

                let mut ctx = RenderContext::new();

                let mut comps = ctx.custom_components.to_vec();
                comps.push(component);

                ctx.custom_components = comps.as_slice();
                let root = ast_mdx(markdown, &ctx).unwrap();

                assert_str_eq!(
                    root.debug_string().unwrap(),
                    indoc! { r#"
                    <a else class={d-button} target={_self} href={https://example.com} data-variant={primary} data-size={md} data-width={fit-content}>
                        <Text>
                            Hi
                        </Text>
                    </a>
                    <Paragraph>
                        <Text>
                            Hi
                        </Text>
                    </Paragraph>
                    "# }
                );
            }
        }
    }

    #[test]
    fn ignores_extra_columns_in_tables_within_ast() {
        let markdown = indoc! {r#"
        |foo|
        |---|
        |bar| BUG |
        "#};

        let ctx = RenderContext::new();
        let root = ast(markdown, &ctx).unwrap();

        let table = root.children.first().unwrap();
        assert!(matches!(table.kind, NodeKind::Table { .. }));

        assert_eq!(table.children.len(), 2);

        assert_eq!(table.children[0].children.len(), 1);
        // The first body row should not have that `BUG` child
        assert_eq!(table.children[1].children.len(), 1);
    }

    #[test]
    fn adds_extra_columns_in_tables_within_ast() {
        let markdown = indoc! {r#"
        |foo| bar |
        |---|-----|
        |bar|
        "#};

        let ctx = RenderContext::new();
        let root = ast(markdown, &ctx).unwrap();

        let table = root.children.first().unwrap();
        assert!(matches!(table.kind, NodeKind::Table { .. }));

        assert_eq!(table.children.len(), 2);

        assert_eq!(table.children[0].children.len(), 2);
        // The first body row should have an extra child generated
        assert_eq!(table.children[1].children.len(), 2);
    }

    #[test]
    fn inline_math_not_parsed_in_old_renderer() {
        let markdown = indoc! {r#"
        $a$
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    $a$
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn math_not_parsed_in_old_renderer() {
        let markdown = indoc! {r#"
        $$
        a
        $$
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    $$
                    a
                    $$
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn yaml_frontmatter_not_parsed() {
        // NOTE: We handle frontmatter _outside_ the Markdown parser.
        // See crate::frontmatter module.
        let markdown = indoc! {r#"
        ---
        foo: true
        ---
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <ThematicBreak />
            <Heading2>
                <Text>
                    foo: true
                </Text>
            </Heading2>
            "# }
        );
    }

    #[test]
    fn toml_frontmatter_not_parsed() {
        let markdown = indoc! {r#"
        +++
        foo = true
        +++
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    +++
                    foo = true
                    +++
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn mdx_flow_expression_not_parsed() {
        let markdown = indoc! {r#"
        {a}
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    {a}
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn mdx_text_expression_not_parsed() {
        let markdown = indoc! {r#"
        a {b}
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
        <Paragraph>
            <Text>
                a {b}
            </Text>
        </Paragraph>
        "# }
        );
    }

    #[test]
    fn mdx_import_statements_not_parsed() {
        let markdown = indoc! {r#"
        import a from 'b'
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    import a from 'b'
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn footnote_not_supported_and_are_interpreted_as_definitions_and_references() {
        let markdown = indoc! {r#"
        Note [^1]

        [^1]: Foo
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    Note
                </Text>
                <Link url={Foo}>
                    <Text>
                        ^1
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn image_references() {
        let markdown = indoc! {r#"
        [bar]: /url

        ![foo][bar]
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Image url={/url} alt={foo} />
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn link_references() {
        let markdown = indoc! {r#"
        [example] [other **text**][example] [example][]

        [example]: https://www.google.com
        "#};

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Link url={https://www.google.com}>
                    <Text>
                        example
                    </Text>
                </Link>
                <Text>

                </Text>
                <Link url={https://www.google.com}>
                    <Text>
                        other
                    </Text>
                    <Strong>
                        <Text>
                            text
                        </Text>
                    </Strong>
                </Link>
                <Text>

                </Text>
                <Link url={https://www.google.com}>
                    <Text>
                        example
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn break_token() {
        let markdown = "foo  \nbar";

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Text>
                    foo
                </Text>
                <Break />
                <Text>
                    bar
                </Text>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn strike_through() {
        let markdown = "~~gone~~";

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Delete>
                    <Text>
                        gone
                    </Text>
                </Delete>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn prefix_asset_urls() {
        let markdown = "![img](/_assets/foo.jpg)";

        let options = RenderOptions {
            prefix_asset_urls: Some("/bar".to_owned()),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Image url={/bar/_assets/foo.jpg} alt={img} />
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn does_not_prefix_remote_image_urls() {
        let markdown = "![an image](https://example.com/foo.png)";

        let options = RenderOptions {
            prefix_asset_urls: Some("/bar".to_owned()),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Image url={https://example.com/foo.png} alt={an image} />
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn webbify_internal_urls() {
        let markdown = "[an link](/foo.md)";

        let options = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Link url={/foo}>
                    <Text>
                        an link
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn ignore_webbifying_non_markdown_links() {
        let markdown = "[an link](/foo)";

        let options = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Link url={/foo}>
                    <Text>
                        an link
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn replace_link_urls() {
        let markdown = "[an link](/foo)";

        let options = RenderOptions {
            link_rewrites: HashMap::from([("/foo".to_string(), "/bar".to_string())]),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Link url={/bar}>
                    <Text>
                        an link
                    </Text>
                </Link>
            </Paragraph>
            "#}
        );
    }

    #[test]
    fn replace_image_urls() {
        let markdown = "![an image](/foo)";

        let options = RenderOptions {
            link_rewrites: HashMap::from([("/foo".to_string(), "/bar".to_string())]),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Image url={/bar} alt={an image} />
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn adds_url_path_prefix() {
        let markdown = "[an link](/foo)";

        let options = RenderOptions {
            prefix_link_urls: Some("/fizz".to_owned()),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! {r#"
            <Paragraph>
                <Link url={/fizz/foo}>
                    <Text>
                        an link
                    </Text>
                </Link>
            </Paragraph>
            "#}
        );
    }

    #[test]
    fn does_not_add_url_prefix_if_matching_url_rewrite() {
        let markdown = "[an link](/foo)";

        let options = RenderOptions {
            prefix_asset_urls: Some("/fizz".to_owned()),
            link_rewrites: HashMap::from([("/foo".to_string(), "/bar".to_string())]),
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Link url={/bar}>
                    <Text>
                        an link
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn does_not_prefix_remote_urls() {
        let markdown = "[external link](https://www.example.com)";

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            &html,
            indoc! { r#"
            <Paragraph>
                <Link url={https://www.example.com}>
                    <Text>
                        external link
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn adds_slugs_to_heading_ast_nodes() {
        let markdown = indoc! {r#"
        ## Foo bar baz

        ## Foo bar baz

        ## Other
        "#};

        let ctx = RenderContext::new();
        let root = ast(markdown, &ctx).unwrap();

        let heading = &root.children[0];

        match &heading.kind {
            NodeKind::Heading { slug, .. } => {
                assert_eq!(slug, "foo-bar-baz");
            }
            _ => panic!("Not a heading: {:#?}", heading),
        }

        let heading = &root.children[1];

        match &heading.kind {
            NodeKind::Heading { slug, .. } => {
                assert_eq!(slug, "foo-bar-baz-1");
            }
            _ => panic!("Not a heading: {:#?}", heading),
        }

        let heading = &root.children[2];

        match &heading.kind {
            NodeKind::Heading { slug, .. } => {
                assert_eq!(slug, "other");
            }
            _ => panic!("Not a heading: {:#?}", heading),
        }
    }

    #[test]
    fn allows_some_inline_ast() {
        let markdown = "<strong>Very bold</strong>";

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Paragraph>
                <strong>
                <Text>
                    Very bold
                </Text>
                </strong>
            </Paragraph>
            "# }
        );
        // format!("<p>{}</p>\n", markdown)
    }

    #[test]
    fn allows_input_fields_with_type_radio() {
        let markdown = "<input type='radio'>";

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <input type='radio'>
            "# }
        );
    }

    #[test]
    fn allows_images_with_src() {
        let markdown = "![A cat](https://www.example.com/cat.jpg)";

        let ctx = RenderContext::new();

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Paragraph>
                <Image url={https://www.example.com/cat.jpg} alt={A cat} />
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn webbifies_urls_consistently_with_proper_slugs() {
        let markdown = "[A link](/föö/bär.md)";

        let options = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_options(&options);

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Paragraph>
                <Link url={/foo/bar}>
                    <Text>
                        A link
                    </Text>
                </Link>
            </Paragraph>
            "# }
        );
    }

    #[test]
    fn expands_relative_paths_in_local_links() {
        let markdown = "[relative link](../other_file.md)";

        let mut ctx = RenderContext::new();
        ctx.with_url_base_by_page_uri("/parent/child/current.md");

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
        <Paragraph>
            <Link url={/parent/other_file.md}>
                <Text>
                    relative link
                </Text>
            </Link>
        </Paragraph>
        "# }
        );

        // assert!(html.contains("<a href=\"/parent/other_file.md\">relative link</a>"));
    }

    #[test]
    fn expands_relative_paths_in_local_links_even_with_link_prefixes() {
        let markdown = "[relative link](../other_file.md)";

        let options = RenderOptions {
            // This is what the desktop does
            prefix_link_urls: Some("#/project".to_owned()),
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.with_options(&options);
        ctx.with_url_base_by_page_uri("/parent/child/current.md");

        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
              <Paragraph>
                  <Link url={#/project/parent/other_file.md}>
                      <Text>
                          relative link
                      </Text>
                  </Link>
              </Paragraph>
            "#
            }
        );
    }

    #[test]
    fn code_with_title() {
        let markdown = indoc! {
          r#"
          ```js title=foo
          const foo = "bar";
          ```
          "#
        };

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Code language={js} title={foo} label={Js} raw={false} show_whitespace={false}>
                const foo = "bar";
            </Code>
            "#
            }
        );
    }

    #[test]
    fn code_with_raw() {
        let markdown = indoc! {
          r#"
          ```js raw
          const foo = "bar";
          ```
          "#
        };

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Code language={js} label={Js} raw={true} show_whitespace={false}>
                const foo = "bar";
            </Code>
            "#
            }
        );
    }

    #[test]
    fn code_with_raw_arbitrary_value() {
        let markdown = indoc! {
          r#"
          ```js raw="foobar"
          const foo = "bar";
          ```
          "#
        };

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Code language={js} label={Js} raw={false} show_whitespace={false}>
                const foo = "bar";
            </Code>
            "#
            }
        );
    }

    #[test]
    fn code_with_show_whitespace() {
        let markdown = indoc! {
          r#"
          ```js show-whitespace
          const foo = "bar";
          ```
          "#
        };

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Code language={js} label={Js} raw={false} show_whitespace={true}>
                const foo = "bar";
            </Code>
            "#
            }
        );
    }

    #[test]
    fn code_with_show_whitespace_arbitrary_value() {
        let markdown = indoc! {
          r#"
          ```js show-whitespace="FASD"
          const foo = "bar";
          ```
          "#
        };

        let ctx = RenderContext::new();
        let html = ast(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Code language={js} label={Js} raw={false} show_whitespace={false}>
                const foo = "bar";
            </Code>
            "#
            }
        );
    }

    #[test]
    fn math() {
        let markdown = indoc! {
          r#"
          $$
          (a - b)^2
          $$
          "#
        };

        let ctx = RenderContext::new();
        let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Math display_mode={false}>
                (a - b)^2
            </Math>
            "#
            }
        );
    }

    #[test]
    fn math_set_display_mode() {
        let markdown = indoc! {
          r#"
          $$ display_mode=true
          (a - b)^2
          $$
          "#
        };

        let ctx = RenderContext::new();
        let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Math display_mode={true}>
                (a - b)^2
            </Math>
            "#
            }
        );
    }

    #[test]
    fn inline_math() {
        let markdown = indoc! {
          r#"
          $$(a - b)^2$$
          "#
        };

        let ctx = RenderContext::new();
        let html = ast_mdx(markdown, &ctx).unwrap().debug_string().unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Paragraph>
                <InlineMath>
                    (a - b)^2
                </InlineMath>
            </Paragraph>
            "#
            }
        );
    }

    mod fault_tolerant_parser {
        use super::*;

        #[test]
        fn can_continue_after_unexpected_closing_tag() {
            let markdown = indoc! {r#"
            </Box>

            Hello
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();
            let ast = result.0.unwrap();
            let errors = result.1;

            assert_eq!(errors.len(), 1);

            assert_str_eq!(
                ast.debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                </Paragraph>
                "# }
            );
        }

        #[test]
        fn can_continue_after_multiple_unexpected_closing_tags() {
            let markdown = indoc! {r#"
            </Box>

            </Box>

            Hello
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let ast = result.0.unwrap();
            let errors = result.1;

            assert_eq!(errors.len(), 2);

            assert_str_eq!(
                ast.debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                </Paragraph>
                "# }
            );
        }

        #[test]
        fn can_handle_valid_markdown_between_unexpected_closing_tags() {
            let markdown = indoc! {r#"
            </Box>

            Hello

            </Box>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let ast = result.0.unwrap();
            let errors = result.1;

            assert_eq!(errors.len(), 2);

            assert_str_eq!(
                ast.debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Text>
                        Hello
                    </Text>
                </Paragraph>
                "# }
            );
        }

        #[test]
        fn can_compute_the_actual_positions_of_the_ast_nodes() {
            let markdown = indoc! {r#"
            </Box>

            Hello

            </Box>

            World
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let ast = result.0.unwrap();

            assert_eq!(ast.children.len(), 2);
            assert_eq!(ast.children[0].pos.start.row, 3);
            assert_eq!(ast.children[0].pos.start.col, 1);
            assert_eq!(ast.children[0].pos.start.byte_offset, 8);
            assert_eq!(ast.children[1].pos.start.row, 7);
            assert_eq!(ast.children[1].pos.start.col, 1);
            assert_eq!(ast.children[1].pos.start.byte_offset, 23);
        }

        #[test]
        fn can_compute_the_actual_positions_of_the_errors() {
            let markdown = indoc! {r#"
            </Box>

            Hello

            </Box>

            World
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let error1 = &result.1[0];
            assert_eq!(error1.position.as_ref().unwrap().start.row, 1);
            assert_eq!(error1.position.as_ref().unwrap().start.col, 1);
            assert_eq!(error1.position.as_ref().unwrap().start.byte_offset, 0);

            let error2 = &result.1[1];
            assert_eq!(error2.position.as_ref().unwrap().start.row, 5);
            assert_eq!(error2.position.as_ref().unwrap().start.col, 1);
            assert_eq!(error2.position.as_ref().unwrap().start.byte_offset, 15);
        }

        #[test]
        fn can_handle_empty_tag() {
            let markdown = indoc! {r#"
            Hello

            <>

            World
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let ast = result.0.unwrap();

            assert_eq!(ast.children.len(), 2);
            assert_eq!(ast.children[0].pos.start.row, 1);
            assert_eq!(ast.children[0].pos.start.col, 1);
            assert_eq!(ast.children[0].pos.start.byte_offset, 0);
            assert_eq!(ast.children[1].pos.start.row, 5);
            assert_eq!(ast.children[1].pos.start.col, 1);
            assert_eq!(ast.children[1].pos.start.byte_offset, 11);
        }

        #[test]
        fn can_handle_empty_unmatching_closing_tag() {
            let markdown = indoc! {r#"
            Hello

            <Card>

            World

            </>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let ast = result.0.unwrap();

            assert_eq!(ast.children.len(), 2);
            assert_eq!(ast.children[0].pos.start.row, 1);
            assert_eq!(ast.children[0].pos.start.col, 1);
            assert_eq!(ast.children[0].pos.start.byte_offset, 0);
            assert_eq!(ast.children[1].pos.start.row, 5);
            assert_eq!(ast.children[1].pos.start.col, 1);
            assert_eq!(ast.children[1].pos.start.byte_offset, 15);
        }

        #[test]
        fn bug_can_compute_positions_of_multiple_errors_correctly() {
            let markdown = indoc! {r#"
            Hello

            <Button href="

            World

            </>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let ast = result.0.unwrap();

            assert_eq!(ast.children.len(), 2);
            assert_eq!(ast.children[0].pos.start.row, 1);
            assert_eq!(ast.children[0].pos.start.col, 1);
            assert_eq!(ast.children[0].pos.start.byte_offset, 0);
            assert_eq!(ast.children[1].pos.start.row, 5);
            assert_eq!(ast.children[1].pos.start.col, 1);
            assert_eq!(ast.children[1].pos.start.byte_offset, 23);

            let errors = result.1;
            assert_eq!(errors.len(), 2);
            assert_eq!(errors[0].position.as_ref().unwrap().start.row, 3);
            assert_eq!(errors[0].position.as_ref().unwrap().start.col, 15);
            assert_eq!(errors[0].position.as_ref().unwrap().start.byte_offset, 21);
            assert_eq!(errors[0].position.as_ref().unwrap().end.row, 3);
            assert_eq!(errors[0].position.as_ref().unwrap().end.col, 15);
            assert_eq!(errors[0].position.as_ref().unwrap().end.byte_offset, 21);

            assert_eq!(errors[1].position.as_ref().unwrap().start.row, 7);
            assert_eq!(errors[1].position.as_ref().unwrap().start.col, 1);
            assert_eq!(errors[1].position.as_ref().unwrap().start.byte_offset, 30);
            assert_eq!(errors[1].position.as_ref().unwrap().end.row, 7);
            assert_eq!(errors[1].position.as_ref().unwrap().end.col, 4);
            assert_eq!(errors[1].position.as_ref().unwrap().end.byte_offset, 33);
        }

        #[test]
        fn bug_can_handle_component_attributes_without_values() {
            let markdown = indoc! {r#"
            <Button href>boom!</Button>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let errors = result.1;
            assert_eq!(errors.len(), 1);
        }

        #[test]
        fn can_handle_non_ascii_characters() {
            let markdown = indoc! {r#"
            Create Account ›
            Learn More ›

            <>

            Read more ›
            "#
            };

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let errors = result.1;
            assert_eq!(errors.len(), 1);
            assert_eq!(errors[0].position.as_ref().unwrap().start.row, 4);
            assert_eq!(errors[0].position.as_ref().unwrap().start.col, 1);
            assert_eq!(errors[0].position.as_ref().unwrap().end.row, 4);
            assert_eq!(errors[0].position.as_ref().unwrap().end.col, 1);
        }

        #[test]
        fn can_handle_non_ascii_characters_2() {
            let markdown = indoc! {r#"
            Create Account ›

            🔥 <Foo></Bar> 🧯

            Learn More ›
            "#
            };

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx).unwrap_err();

            let errors = result.1;
            assert_eq!(errors.len(), 1);
            let pos = &errors[0].position.as_ref().unwrap();

            assert_eq!(
                &markdown[pos.start.byte_offset..pos.end.byte_offset],
                "</Bar>"
            );

            assert_eq!(pos.start.row, 3, "Wrong start row");
            assert_eq!(pos.start.col, 8, "Wrong start col");
            assert_eq!(pos.end.row, 3, "Wrong end row");
            assert_eq!(pos.end.col, 14, "Wrong end col");
        }

        #[test]
        fn bug_stack_overflow() {
            let markdown = indoc! {r#"
            <Card>break
                Foo
            </Card>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx);
            // Should not stack overflow
            assert!(result.is_err());
        }

        #[test]
        fn bug_stack_overflow_2() {
            let markdown = indoc! {r#"
            <Flex gap="1">
              <Button href="https://dashboard.doctave.com/" size="lg">
                Create Account ›
              </Button>

              <Button>A

              <Button href="/concepts/introduction.md" size="lg" variant="secondary">
                 Learn More ›
              </Button>
            </Flex>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx);
            // Should not stack overflow
            assert!(result.is_err());
        }

        #[test]
        fn bug_error_renderer_integer_subtract_overflow() {
            // DOC-1298
            let markdown = indoc! {r#"
            <Box asd=""></Box>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx);
            // Should not integer overflow
            assert!(result.is_err());
        }

        #[test]
        fn bug_error_message_for_invalid_conditional() {
            let markdown = indoc! {r#"
            <Box if={true}></Box>
            asd
            <Box else></Box>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx);
            assert!(result.is_err());
        }

        #[test]
        fn bug_underflow_in_error_renderer() {
            // DOC-1297
            let markdown = indoc! {r#"
            <Box gap="1">
            </Box>asd
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx);
            assert!(result.is_err());
            assert_eq!(
                &result.unwrap_err().1[0].description,
                indoc! {r#"
                Expected the closing tag `</Box>` either before the start of `Paragraph`, or another opening tag after that start

                    1 │ <Box gap="1">
                    2 │ </Box>asd
                        ▲
                        │
                        └─ Closing tag
                        ╷
                        └─ Opened tag

                "#}
            );
        }

        #[test]
        fn bug_unclosed_expression() {
            // DOC-1321
            //
            // This is nasty, because our fault-tolerant parser cannot recover from
            // this error gracefully. So instead of panicking, we're going to try to
            // return an empty result.
            let markdown = indoc! {r#"
            <Tabs>
                <Tab title={
                </Tab>
            </Tabs>
            "#};

            let ctx = RenderContext::new();
            let result = ast_mdx_fault_tolerant(markdown, &ctx);
            assert!(result.is_err());
        }
    }

    mod error_positions {
        use super::*;

        #[test]
        fn unexpected_closing_tag() {
            let markdown = indoc! {r#"
            Foo

            </Box>

            Bar
            "#};

            let ctx = RenderContext::new();
            let error = ast_mdx(markdown, &ctx).unwrap_err();

            let position = error.position.as_ref().unwrap();

            assert_eq!(position.start.row, 3);
            assert_eq!(position.start.col, 1);
            assert_eq!(position.start.byte_offset, 5);

            assert_eq!(position.end.row, 3);
            assert_eq!(position.end.col, 7);
            assert_eq!(position.end.byte_offset, 11);

            assert_eq!(
                &markdown[position.start.byte_offset..position.end.byte_offset],
                "</Box>"
            );
        }

        #[test]
        fn wrong_closing_tag() {
            let markdown = indoc! {r#"
            Foo

            <Box>

            Bar

            </Bunny>

            Baz
            "#};

            let ctx = RenderContext::new();
            let error = ast_mdx(markdown, &ctx).unwrap_err();

            let position = error.position.as_ref().unwrap();

            assert_eq!(position.start.row, 7);
            assert_eq!(position.start.col, 1);
            assert_eq!(position.start.byte_offset, 17);

            assert_eq!(position.end.row, 7);
            assert_eq!(position.end.col, 9);
            assert_eq!(position.end.byte_offset, 25);

            assert_eq!(
                &markdown[position.start.byte_offset..position.end.byte_offset],
                "</Bunny>"
            );
        }
    }
}
