use std::collections::HashMap;
use std::path::{Path, PathBuf};

use crate::frontmatter::{Frontmatter, PageWidth};
use crate::markdown::{Node, NodeKind};
use crate::page_kind::OutgoingLink;
use crate::render_context::{FileContext, RenderContext};
use crate::utils::capitalize;
use crate::{frontmatter, markdown, Error, Result};

#[cfg(test)]
use ts_rs::TS;

#[derive(Clone)]
pub(crate) struct MarkdownPage {
    pub path: PathBuf,
    pub uri_path: String,
    pub content: String,
}

impl MarkdownPage {
    pub(crate) fn new(path: &Path, content: Vec<u8>) -> MarkdownPage {
        MarkdownPage {
            path: path.to_owned(),
            uri_path: crate::fs_to_uri_path(path),
            content: String::from_utf8(content).expect("Invalid UTF8 sequence"),
        }
    }

    pub fn title(&self) -> Result<Option<String>> {
        self.frontmatter()
            .map(|f| f.title.or(Self::titelize(&self.path)))
    }

    pub fn description(&self) -> Result<Option<String>> {
        self.frontmatter().map(|f| f.meta.description)
    }

    pub fn hidden_from_search(&self) -> Result<bool> {
        self.frontmatter().map(|f| f.search.hidden)
    }

    pub fn show_breadcrumbs(&self) -> bool {
        self.frontmatter().map(|f| f.breadcrumbs).unwrap_or(true)
    }

    pub fn hide_side_table_of_contents(&self) -> bool {
        self.frontmatter()
            .map(|f| !f.toc || f.hide_side_table_of_contents)
            .unwrap_or(false)
    }

    pub fn hide_navigation(&self) -> bool {
        self.frontmatter()
            .map(|f| !f.navigation || f.hide_navigation)
            .unwrap_or(false)
    }

    pub fn page_width(&self) -> PageWidth {
        self.frontmatter().map(|f| f.page_width).unwrap_or_default()
    }

    pub fn frontmatter(&self) -> Result<Frontmatter> {
        frontmatter::parse(&self.content).map_err(|e| Error {
            code: Error::INVALID_FRONTMATTER,
            message: "Invalid YAML syntax in frontmatter".to_owned(),
            description: e,
            file: Some(self.path.clone()),
            position: None,
        })
    }

    fn frontmatter_lines_offset(&self) -> usize {
        self.content[..self.frontmatter_chars_offset()]
            .lines()
            .count()
    }

    fn frontmatter_chars_offset(&self) -> usize {
        frontmatter::end_pos(&self.content)
    }

    pub(crate) fn outgoing_links(&self, ctx: &mut RenderContext) -> Result<Vec<OutgoingLink>> {
        ctx.with_url_base_by_fs_path(&self.path);

        // NOTE(Nik): We want the ast _without the expanding relative links_.
        // We will expand the links below, once we've gathered the links, and
        // this lets us give the user the actual URI that they've written in
        // the markdown file as the error message, instead of the expanded version.
        if self.experimental_template_rendered_enabled(ctx.settings.is_v2()) {
            markdown::parser::extract_links(&self.content, ctx)
        } else {
            let full_content = self.apply_partials(ctx)?;
            markdown::parser::extract_links(&full_content, ctx)
        }
    }

    pub(crate) fn asset_links(&self, ctx: &mut RenderContext) -> Result<Vec<OutgoingLink>> {
        ctx.with_url_base_by_fs_path(&self.path);

        if self.experimental_template_rendered_enabled(ctx.settings.is_v2()) {
            markdown::parser::extract_asset_links(&self.content, ctx)
        } else {
            let full_content = self.apply_partials(ctx)?;
            markdown::parser::extract_asset_links(&full_content, ctx)
        }
    }

    pub(crate) fn external_links(&self, ctx: &mut RenderContext) -> Result<Vec<String>> {
        ctx.with_url_base_by_page_uri(&self.uri_path);

        if self.experimental_template_rendered_enabled(ctx.settings.is_v2()) {
            markdown::parser::extract_external_links(&self.content)
        } else {
            let full_content = self.apply_partials(ctx)?;
            markdown::parser::extract_external_links(&full_content)
        }
    }

    fn titelize(path: &Path) -> Option<String> {
        let t = path.with_extension("");

        match t.file_name().and_then(|s| s.to_str()) {
            None => None,
            Some("README") => t.parent().and_then(Self::titelize),
            Some(other) => Some(capitalize(other)),
        }
        .map(|s| s.replace(['-', '_'], " "))
    }

    pub fn apply_partials(&self, ctx: &mut RenderContext) -> crate::Result<String> {
        let mut globals = liquid::Object::new();

        let preferences = ctx
            .settings
            .user_preferences()
            .iter()
            .map(|(k, p)| (k.to_string(), p.default.to_string()))
            .chain(ctx.options.user_preferences.clone())
            .collect::<HashMap<_, _>>();

        // We get a type error without this clone on preferences? I think it has something to do with
        // how the marco is built internally.
        #[allow(clippy::redundant_clone)]
        globals.insert(
            "DOCTAVE".into(),
            liquid_core::Value::Object(liquid::object!({
                "user_preferences": liquid::model::Value::Object(liquid::object!(preferences.clone())),
            }))
        );

        let parsed = ctx
            .liquid_parser
            .parse(frontmatter::without(&self.content))?;

        parsed.render(&globals).map_err(|e| e.into())
    }

    pub fn experimental_template_rendered_enabled(&self, is_v2: bool) -> bool {
        if let Ok(fm) = self.frontmatter() {
            fm.experimental.v2_templates.unwrap_or(is_v2)
        } else {
            is_v2
        }
    }

    pub fn ast(&self, ctx: &mut RenderContext) -> crate::Result<Node> {
        ctx.with_url_base_by_fs_path(&self.path);
        ctx.with_file_context(FileContext::new(
            self.frontmatter_lines_offset(),
            self.frontmatter_chars_offset(),
            self.path.clone(),
        ));

        if self.experimental_template_rendered_enabled(ctx.settings.is_v2()) {
            markdown::ast_mdx(frontmatter::without(&self.content), ctx)
        } else {
            let full = self.apply_partials(ctx)?;

            markdown::ast(&full, ctx)
        }
    }

    pub fn on_this_page_headings(&self, ctx: &mut RenderContext) -> Vec<OnThisPageHeading> {
        let mut headings = vec![];

        let ast = if let Ok(a) = self.ast(ctx) {
            a
        } else {
            return vec![];
        };

        for node in ast.children {
            if let NodeKind::Heading { level, ref slug } = node.kind {
                if level > 1 && level < 5 {
                    let title = node.inner_text();

                    headings.push(OnThisPageHeading {
                        level: level - 1,
                        anchor: slug.to_owned(),
                        title,
                    })
                }
            }
        }

        headings
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct OnThisPageHeading {
    pub level: u8,
    pub title: String,
    pub anchor: String,
}

impl std::fmt::Debug for MarkdownPage {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::result::Result<(), std::fmt::Error> {
        fmt.debug_struct("Page")
            .field("content", &self.content)
            .field("path", &self.path)
            .field("uri_path", &self.uri_path)
            .finish()?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{renderable_ast::NodeKind, settings::Settings, RenderOptions};
    use pretty_assertions::{assert_eq, assert_str_eq};
    use std::path::Path;

    #[test]
    fn use_experimental_mdx_renderer_with_frontmatter_option() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---
            experimental:
              v2_templates: true
            ---
            <Box />
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let root = page.ast(&mut ctx).unwrap();

        assert!(
            matches!(root.children[0].kind, NodeKind::Box { .. }),
            "Not parsed with experimental parser. Got: {:#?}",
            root
        );
    }

    #[test]
    fn experimental_renderer_offsets_error_lines_by_frontmatter_length() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---
            experimental:
              v2_templates: true
            ---

            Hello

            { 1 + true }

            bar
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let err = page.ast(&mut ctx).unwrap_err();

        assert_eq!(
            &err.description,
            indoc! {r#"
            Cannot apply operation `+` on values `1` with type `number` and `true` with type `bool`

                7 │
                8 │ { 1 + true }
                      ▲▲▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn experimental_renderer_offsets_error_lines_by_frontmatter_length_for_ast() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---
            experimental:
              v2_templates: true
            ---

            Hello

            { 1 + true }

            bar
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let err = page.ast(&mut ctx).unwrap_err();

        assert_eq!(
            &err.description,
            indoc! {r#"
            Cannot apply operation `+` on values `1` with type `number` and `true` with type `bool`

                7 │
                8 │ { 1 + true }
                      ▲▲▲▲▲▲▲▲

            "#}
        );
    }

    #[test]
    fn defaults_to_old_renderer() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            <Foo />
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let root = page.ast(&mut ctx).unwrap();

        if let Node {
            kind: NodeKind::HtmlTag { value },
            ..
        } = &root.children[0]
        {
            assert_eq!(value, "<Foo />");
        } else {
            panic!("Not parsed with old parser by default. Got: {:#?}", root);
        }
    }

    #[test]
    fn on_this_page_headings() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
        # Main heading

        ## Secondary header 1

        ### Level 3 header 1

        #### Level 4 header 1

        ##### IGNORE ME

        ## Secondary header 2

        ### Level 3

        #### Level 4 header 2

        ##### IGNORE ME

        ### Level 3
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let headings = page.on_this_page_headings(&mut ctx);

        assert_eq!(
            headings[0],
            OnThisPageHeading {
                level: 1,
                title: "Secondary header 1".to_owned(),
                anchor: "secondary-header-1".to_string()
            }
        );
        assert_eq!(
            headings[1],
            OnThisPageHeading {
                level: 2,
                title: "Level 3 header 1".to_owned(),
                anchor: "level-3-header-1".to_string()
            }
        );
        assert_eq!(
            headings[2],
            OnThisPageHeading {
                level: 3,
                title: "Level 4 header 1".to_owned(),
                anchor: "level-4-header-1".to_string()
            }
        );
        assert_eq!(
            headings[3],
            OnThisPageHeading {
                level: 1,
                title: "Secondary header 2".to_owned(),
                anchor: "secondary-header-2".to_string()
            }
        );
        assert_eq!(
            headings[4],
            OnThisPageHeading {
                level: 2,
                title: "Level 3".to_owned(),
                anchor: "level-3".to_string()
            }
        );
        assert_eq!(
            headings[5],
            OnThisPageHeading {
                level: 3,
                title: "Level 4 header 2".to_owned(),
                anchor: "level-4-header-2".to_string()
            }
        );
        assert_eq!(
            headings[6],
            OnThisPageHeading {
                level: 2,
                title: "Level 3".to_owned(),
                anchor: "level-3-1".to_string()
            }
        );
    }

    #[test]
    fn on_this_page_headings_bold_italic_code() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
        ## With **bold** and _italic_ and `code`

        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let headings = page.on_this_page_headings(&mut ctx);

        assert_eq!(
            headings[0],
            OnThisPageHeading {
                level: 1,
                title: "With bold and italic and code".to_owned(),
                anchor: "with-bold-and-italic-and-code".to_string()
            }
        );
    }

    #[test]
    fn does_not_explode_if_the_page_has_an_empty_frontmatter() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---
            ---
            # Hi
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let html = page
            .ast(&mut ctx)
            .expect("failed to render")
            .debug_string()
            .unwrap();

        assert_str_eq!(
            html,
            indoc! { r#"
            <Heading1>
                <Text>
                    Hi
                </Text>
            </Heading1>
            "# }
        );
    }

    #[test]
    fn does_not_explode_when_asking_for_title_if_the_page_has_an_empty_frontmatter() {
        let page = MarkdownPage::new(
            Path::new("Foo/README.md"),
            indoc! {"
            ---
            ---
            # Hi
        "}
            .as_bytes()
            .to_owned(),
        );

        let title = page.title().expect("failed to title");

        assert_eq!(title.as_deref(), Some("Foo"));
    }

    #[test]
    fn bug_does_not_include_frontmatter_as_a_heading() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---
            title: Wasd
            ---
        "}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let headings = page.on_this_page_headings(&mut ctx);

        assert_eq!(headings.len(), 0);
    }

    #[test]
    fn exposes_user_preferences_as_liquid_variable() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {r#"
            {% if DOCTAVE.user_preferences.foo == "bar" %}
            BAR
            {% endif %}
            "#}
            .as_bytes()
            .to_owned(),
        );

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("foo".to_string(), "bar".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);
        let rendered = page.ast(&mut ctx).unwrap().debug_string().unwrap();

        assert!(rendered.contains("BAR"));
    }

    #[test]
    fn exposes_default_user_preferences_as_liquid_variable() {
        let settings: Settings = Settings::parse(indoc! {r#"
        ---
        title: Example
        user_preferences:
          foo:
            label: Plan
            default: bar
            values:
              - bar
              - something else
        "#})
        .unwrap();

        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {r#"
            {% if DOCTAVE.user_preferences.foo == "bar" %}
            BAR
            {% endif %}

            "#}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        ctx.with_settings(&settings);
        let rendered = page.ast(&mut ctx).unwrap().debug_string().unwrap();

        assert!(rendered.contains("BAR"));
    }

    #[test]
    fn it_correctly_generates_outgoing_links_for_root_readme_md_files_with_relative_links() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {r#"
            # I'm the root README.md file

            [link](fizz/buzz.md)
            "#}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let links = page.outgoing_links(&mut ctx).unwrap();

        assert_eq!(links[0].expanded_uri.as_deref(), Some("/fizz/buzz.md"));
    }

    #[test]
    fn it_correctly_generates_outgoing_links_for_nested_files() {
        let page = MarkdownPage::new(
            Path::new("foo/bar.md"),
            indoc! {r#"
            # I'm the root README.md file

            [link](fizz/buzz.md)
            "#}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let links = page.outgoing_links(&mut ctx).unwrap();

        assert_eq!(links[0].expanded_uri.as_deref(), Some("/foo/fizz/buzz.md"));
    }

    #[test]
    fn it_correctly_generates_outgoing_links_for_nested_readme_md_files_with_relative_links() {
        let page = MarkdownPage::new(
            Path::new("foo/bar/README.md"),
            indoc! {r#"
            # I'm a nested README.md file

            [link](fizz/buzz.md)
            "#}
            .as_bytes()
            .to_owned(),
        );

        let mut ctx = RenderContext::new();
        let links = page.outgoing_links(&mut ctx).unwrap();

        assert_eq!(
            links[0].expanded_uri.as_deref(),
            Some("/foo/bar/fizz/buzz.md")
        );
    }
}
