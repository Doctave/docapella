use std::path::Path;

use crate::{
    breadcrumb::{self, Breadcrumb},
    frontmatter::PageWidth,
    markdown_page::OnThisPageHeading,
    page_kind::{Ast, OutgoingLink, PageKind},
    render_context::RenderContext,
    Project, RenderOptions, Result,
};

#[derive(Clone, Debug)]
pub struct PageHandle<'a> {
    pub(crate) page: &'a PageKind,
    pub(crate) project: &'a Project,
}

impl PageHandle<'_> {
    pub fn uri_path(&self) -> &str {
        self.page.uri_path()
    }

    pub fn fs_path(&self) -> &Path {
        self.page.fs_path()
    }

    pub fn title(&self) -> Result<Option<String>> {
        self.page.title()
    }

    pub fn description(&self) -> Result<Option<String>> {
        self.page.description()
    }

    pub fn hidden_from_search(&self) -> Result<bool> {
        self.page.hidden_from_search()
    }

    pub fn openapi_tag(&self) -> Option<&str> {
        self.page.openapi_tag()
    }

    pub fn show_breadcrumbs(&self) -> bool {
        match self.page {
            PageKind::Markdown(m) => m.show_breadcrumbs(),
            PageKind::OpenApi(_) => true,
        }
    }

    pub fn breadcrumbs(&self, opts: Option<&RenderOptions>) -> Vec<Breadcrumb> {
        // NOTE: Don't worry about errors here. They'll be reported elsewhere.
        breadcrumb::compute(self.uri_path(), self.project, opts).unwrap_or_default()
    }

    pub fn is_markdown(&self) -> bool {
        matches!(&self.page, PageKind::Markdown(_))
    }

    pub fn is_openapi(&self) -> bool {
        matches!(&self.page, PageKind::OpenApi(_))
    }

    pub fn ast(&self, opts: Option<&RenderOptions>) -> Result<Ast> {
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(opts);
        ctx.with_project(self.project);

        self.page.ast(&mut ctx)
    }

    pub fn hide_side_table_of_contents(&self) -> bool {
        match &self.page {
            PageKind::Markdown(p) => p.hide_side_table_of_contents(),
            _ => false,
        }
    }

    pub fn hide_navigation(&self) -> bool {
        match &self.page {
            PageKind::Markdown(p) => p.hide_navigation(),
            _ => false,
        }
    }

    pub fn page_width(&self) -> PageWidth {
        match &self.page {
            PageKind::Markdown(p) => p.page_width(),
            PageKind::OpenApi(_) => PageWidth::Full,
        }
    }

    pub fn on_this_page_headings(&self, opts: Option<&RenderOptions>) -> Vec<OnThisPageHeading> {
        match &self.page {
            PageKind::Markdown(p) => {
                let mut ctx = RenderContext::new();
                ctx.with_maybe_options(opts);
                ctx.with_project(self.project);

                p.on_this_page_headings(&mut ctx)
            }
            _ => vec![],
        }
    }

    pub(crate) fn outgoing_links(&self, opts: Option<&RenderOptions>) -> Result<Vec<OutgoingLink>> {
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(opts);
        ctx.with_project(self.project);

        self.page.outgoing_links(&mut ctx)
    }

    pub(crate) fn asset_links(&self, opts: Option<&RenderOptions>) -> Result<Vec<OutgoingLink>> {
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(opts);
        ctx.with_project(self.project);

        self.page.asset_links(&mut ctx)
    }

    pub(crate) fn external_links(&self, opts: Option<&RenderOptions>) -> Result<Vec<String>> {
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(opts);
        ctx.with_project(self.project);

        self.page.external_links(&mut ctx)
    }
}
