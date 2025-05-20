use crate::markdown::Node;
use crate::open_api::ast::PageAst;
use crate::utils::capitalize;
use crate::{render_context::RenderContext, MarkdownPage, OpenApiPage, Result};

use std::path::Path;

#[cfg(test)]
use ts_rs::TS;

#[cfg(feature = "rustler")]
use rustler::NifTaggedEnum;

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifTaggedEnum))]
#[serde(tag = "kind", content = "root", rename_all = "snake_case")]
pub enum Ast {
    Markdown(Node),
    OpenApi(PageAst),
}

impl Ast {
    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn as_markdown(&self) -> Option<&Node> {
        match self {
            Self::Markdown(n) => Some(n),
            _ => None,
        }
    }

    #[cfg(test)]
    #[allow(dead_code)]
    pub(crate) fn as_openapi(&self) -> Option<&PageAst> {
        match self {
            Self::OpenApi(n) => Some(n),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) enum PageKind {
    Markdown(MarkdownPage),
    OpenApi(OpenApiPage),
}

#[derive(Debug, Clone, PartialEq)]
/// Return type for the `outgoing_links` function.
pub(crate) struct OutgoingLink {
    pub uri: String,
    /// If the original URI was relative, this will be the
    /// expanded absolut version of the URI.
    pub expanded_uri: Option<String>,
}

impl PageKind {
    pub fn uri_path(&self) -> &str {
        match self {
            Self::Markdown(md) => &md.uri_path,
            Self::OpenApi(oapi) => &oapi.uri_path,
        }
    }

    pub fn fs_path(&self) -> &Path {
        match self {
            Self::Markdown(md) => &md.path,
            Self::OpenApi(oapi) => &oapi.fs_path,
        }
    }

    pub fn title(&self) -> Result<Option<String>> {
        match self {
            Self::Markdown(md) => md.title(),
            Self::OpenApi(oapi) => Ok(oapi.tag().map(capitalize)),
        }
    }

    pub fn description(&self) -> Result<Option<String>> {
        match self {
            Self::Markdown(md) => md.description(),
            Self::OpenApi(_oapi) => Ok(None),
        }
    }

    pub fn hidden_from_search(&self) -> Result<bool> {
        match self {
            Self::Markdown(md) => md.hidden_from_search(),
            Self::OpenApi(_oapi) => Ok(false),
        }
    }

    pub fn openapi_tag(&self) -> Option<&str> {
        match self {
            Self::Markdown(_) => None,
            Self::OpenApi(oapi) => oapi.tag(),
        }
    }

    #[allow(dead_code)]
    pub fn markdown(&self) -> Option<&MarkdownPage> {
        match self {
            Self::Markdown(p) => Some(p),
            Self::OpenApi(_) => None,
        }
    }

    pub fn ast(&self, ctx: &mut RenderContext) -> crate::Result<Ast> {
        let result = match &self {
            Self::Markdown(p) => p.ast(ctx).map(Ast::Markdown),
            Self::OpenApi(o) => o.ast(ctx).map(Ast::OpenApi),
        };

        match result {
            Err(mut e) => {
                e.in_file(self.fs_path());
                Err(e)
            }
            ok => ok,
        }
    }

    /// Lists all the links from the page.
    pub(crate) fn outgoing_links(&self, ctx: &mut RenderContext) -> Result<Vec<OutgoingLink>> {
        match &self {
            Self::Markdown(p) => p.outgoing_links(ctx),
            Self::OpenApi(p) => p.outgoing_links(ctx),
        }
    }

    pub(crate) fn asset_links(&self, ctx: &mut RenderContext) -> Result<Vec<OutgoingLink>> {
        match &self {
            Self::Markdown(p) => p.asset_links(ctx),
            Self::OpenApi(_p) => Ok(vec![]), // TODO
        }
    }

    pub(crate) fn external_links(&self, ctx: &mut RenderContext) -> Result<Vec<String>> {
        match &self {
            Self::Markdown(p) => p.external_links(ctx),
            Self::OpenApi(_) => Ok(vec![]), // TODO,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn markdown_pages_can_have_a_fallback_title_from_their_base_name() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("README.md"),
            String::new().into_bytes(),
        ));

        assert_eq!(page.title(), Ok(None));

        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Foo.md"),
            String::new().into_bytes(),
        ));

        assert_eq!(page.title(), Ok(Some("Foo".to_owned())));

        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("something/README.md"),
            String::new().into_bytes(),
        ));

        assert_eq!(page.title(), Ok(Some("Something".to_owned())));
    }

    #[test]
    fn fallback_titles_get_formatted() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Foo-in-the-Bar.md"),
            String::new().into_bytes(),
        ));

        assert_eq!(page.title(), Ok(Some("Foo in the Bar".to_string())));
    }

    #[test]
    fn fallback_titles_get_formatted_2() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Foo_in_the_Bar.md"),
            String::new().into_bytes(),
        ));

        assert_eq!(page.title(), Ok(Some("Foo in the Bar".to_string())));
    }

    #[test]
    fn fallback_titles_get_capitalized() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("my-case.md"),
            String::new().into_bytes(),
        ));

        assert_eq!(page.title(), Ok(Some("My case".to_string())));
    }

    #[test]
    fn can_override_title_with_frontmatter() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Not-Me.md"),
            indoc! {r#"
            ---
            title: Something else
            ---
            "#}
            .as_bytes()
            .to_owned(),
        ));

        assert_eq!(page.title(), Ok(Some("Something else".to_string())));
    }

    #[test]
    fn title_is_not_capitalized_if_explicitly_set() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Not-Me.md"),
            indoc! {r#"
            ---
            title: lowercase
            ---
            "#}
            .as_bytes()
            .to_owned(),
        ));

        assert_eq!(page.title(), Ok(Some("lowercase".to_string())));
    }

    #[test]
    fn returns_an_error_on_malformed_frontmatter() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Not-Me.md"),
            indoc! {r#"
            ---
            - title: Something else
              asdf
            ---
            "#}
            .as_bytes()
            .to_owned(),
        ));

        let e = page.title().unwrap_err();
        assert_eq!(e.code, crate::Error::INVALID_FRONTMATTER);
    }

    #[test]
    fn seo_description_can_be_set_in_markdown_frontmatter() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Not-Me.md"),
            indoc! {r#"
            ---
            title: Something else
            meta:
              description: Some description
            ---
            "#}
            .as_bytes()
            .to_owned(),
        ));

        assert_eq!(page.description(), Ok(Some("Some description".to_string())));
    }

    #[test]
    fn defaults_to_not_hidden_from_search() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Not-Me.md"),
            indoc! {r#"
            ---
            ---
            "#}
            .as_bytes()
            .to_owned(),
        ));

        assert_eq!(page.hidden_from_search(), Ok(false));
    }

    #[test]
    fn can_be_marked_as_hidden_from_search() {
        let page = PageKind::Markdown(MarkdownPage::new(
            Path::new("Not-Me.md"),
            indoc! {r#"
            ---
            search:
              hidden: true
            ---
            "#}
            .as_bytes()
            .to_owned(),
        ));

        assert_eq!(page.hidden_from_search(), Ok(true));
    }
}
