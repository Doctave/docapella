use crate::open_api;
use crate::open_api::ast::PageAst;
use crate::page_kind::OutgoingLink;
use crate::render_context::RenderContext;

use std::path::PathBuf;

#[derive(Clone)]
pub(crate) struct OpenApiPage {
    pub uri_path: String,
    pub fs_path: PathBuf,
    page: open_api::model::Page,
}

impl OpenApiPage {
    pub(crate) fn new(page: open_api::model::Page) -> Self {
        OpenApiPage {
            uri_path: page.uri_path.clone(),
            fs_path: page.fs_path.clone(),
            page,
        }
    }

    pub fn tag(&self) -> Option<&str> {
        match self.page.tag.name.as_str() {
            "" => None,
            some => Some(some),
        }
    }

    pub fn ast(&self, ctx: &mut RenderContext) -> crate::Result<PageAst> {
        ctx.with_url_base_by_page_uri(self.uri_path.as_str());

        self.page.ast(ctx)
    }

    pub(crate) fn outgoing_links(
        &self,
        ctx: &mut RenderContext,
    ) -> crate::Result<Vec<OutgoingLink>> {
        self.page.outgoing_links(ctx)
    }

    pub(crate) fn operations(&self) -> &[open_api::model::Operation] {
        &self.page.operations
    }

    #[cfg(test)]
    pub(crate) fn get_page(&self) -> &open_api::model::Page {
        &self.page
    }
}

impl std::fmt::Debug for OpenApiPage {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("OpenApiPage")
            .field("uri_path", &self.uri_path)
            .field("page", &self.page)
            .finish()?;

        Ok(())
    }
}
