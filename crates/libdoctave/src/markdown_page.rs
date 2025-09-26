use std::path::{Path, PathBuf};

use crate::frontmatter::{Frontmatter, PageWidth};
use crate::markdown::{Node, NodeKind};
use crate::page_kind::OutgoingLink;
use crate::render_context::{FileContext, RenderContext};
use crate::utils::capitalize;
use crate::{frontmatter, markdown, Error, Result};

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
        markdown::parser::extract_links(&self.content, ctx)
    }

    pub(crate) fn asset_links(&self, ctx: &mut RenderContext) -> Result<Vec<OutgoingLink>> {
        markdown::parser::extract_asset_links(&self.content, ctx)
    }

    pub(crate) fn external_links(&self, ctx: &mut RenderContext) -> Result<Vec<String>> {
        ctx.with_url_base_by_page_uri(&self.uri_path);

        markdown::parser::extract_external_links(&self.content, ctx)
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

    pub fn ast(&self, ctx: &mut RenderContext) -> crate::Result<Node> {
        ctx.with_url_base_by_fs_path(&self.path);
        ctx.with_file_context(FileContext::new(
            self.frontmatter_lines_offset(),
            self.frontmatter_chars_offset(),
            self.path.clone(),
        ));

        markdown::ast_mdx(frontmatter::without(&self.content), ctx)
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
    use pretty_assertions::{assert_eq, assert_str_eq};
    use std::path::Path;

    #[test]
    fn renderer_offsets_error_lines_by_frontmatter_length() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---


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
    fn renderer_offsets_error_lines_by_frontmatter_length_for_ast() {
        let page = MarkdownPage::new(
            Path::new("README.md"),
            indoc! {"
            ---


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
