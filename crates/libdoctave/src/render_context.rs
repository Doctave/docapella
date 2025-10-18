use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use std::{ffi::OsStr, path::PathBuf};

use crate::open_api::model::Components;
use crate::page_kind::PageKind;
use crate::project::Asset;
use crate::Project;
use crate::{markdown::CustomComponentHandle, settings::Settings, RenderOptions, BAKED_COMPONENTS};

/// This struct represents the context for rendering a page.
/// The RenderOptions struct is provided from outside of
/// libdoctave on each render, but this struct can contain
/// contextual information determined _internally_ to libdoctave.
#[derive(Clone)]
pub(crate) struct RenderContext<'a> {
    pub options: &'a RenderOptions,
    pub settings: &'a Settings,
    pub relative_url_base: Option<String>,
    pub file_context: Option<FileContext>,
    pub pages: &'a [PageKind],
    /// Ballad custom components defined in `_components`.
    pub custom_components: &'a [CustomComponentHandle],
    pub assets: &'a [Asset],
    pub openapi_components: &'a HashMap<String, Components>,
    /// Global timestamp for cache busting image URLs
    pub cache_bust_timestamp: String,
}

lazy_static! {
    static ref DEFAULT_OPTS: RenderOptions = RenderOptions::default();
    static ref DEFAULT_SETTINGS: Settings = Settings::default();
    static ref DEFAULT_OPENAPI_COMPONENTS: HashMap<String, Components> = HashMap::default();
}

impl Default for RenderContext<'_> {
    fn default() -> Self {
        let now = SystemTime::now();
        let cache_bust_timestamp = now
            .duration_since(UNIX_EPOCH)
            .expect("Time went backwards")
            .as_millis()
            .to_string();

        RenderContext {
            options: &DEFAULT_OPTS,
            settings: &DEFAULT_SETTINGS,
            pages: &[],
            relative_url_base: None,
            file_context: None,
            custom_components: &BAKED_COMPONENTS,
            assets: &[],
            openapi_components: &DEFAULT_OPENAPI_COMPONENTS,
            cache_bust_timestamp,
        }
    }
}

impl<'a> RenderContext<'a> {
    pub fn new() -> Self {
        Default::default()
    }

    /// Convenience method for setting up a render context from a project.
    ///
    /// Calls the various setters for the context.
    pub fn with_project(&mut self, project: &'a Project) {
        self.with_pages(&project.pages);
        self.with_settings(&project.settings);
        self.with_custom_components(&project.custom_components);
        self.with_assets(&project.assets);
        self.with_openapi_components(&project.open_api_components);
    }

    pub fn with_pages(&mut self, pages: &'a [PageKind]) {
        self.pages = pages;
    }

    pub fn with_settings(&mut self, settings: &'a Settings) {
        self.settings = settings;
    }

    pub fn with_custom_components(&mut self, components: &'a [CustomComponentHandle]) {
        self.custom_components = components;
    }

    pub fn with_assets(&mut self, assets: &'a [Asset]) {
        self.assets = assets;
    }

    pub fn with_openapi_components(&mut self, components: &'a HashMap<String, Components>) {
        self.openapi_components = components;
    }

    pub fn with_file_context(&mut self, file_context: FileContext) {
        self.file_context = Some(file_context);
    }

    #[cfg(test)]
    /// Shorthand for tests where we have an explicit option
    pub fn with_options(&mut self, options: &'a RenderOptions) {
        self.options = options;
    }

    pub fn with_maybe_options(&mut self, options: Option<&'a RenderOptions>) {
        if let Some(opts) = options.as_ref() {
            self.options = opts;
        }
    }

    pub fn with_url_base(&mut self, base: &str) {
        self.relative_url_base = Some(base.trim_end_matches('/').to_string());
    }

    pub fn with_url_base_by_fs_path(&mut self, fs_path: &Path) {
        // Convert the path of the file to a URI
        //
        // We can't just pop the fs_path and then convert to URI, because the folder
        // containing the file may have an ending that looks like a file extension,
        // e.g. `v1.0.2`, which `fs_to_uri_path` would truncate to `v.1.0`
        //
        // So instead, we convert the actual page's path to a URI first.
        let uri = crate::fs_to_uri_path(fs_path);

        let base = if fs_path.file_name() == Some(OsStr::new("README.md")) {
            // Then, if we're at the root, or we were looking at a README.md file,
            // (e.g. in /foo/bar/README.md), then the URI base is just /foo/bar, so
            // return the URI directly
            uri
        } else {
            // Otherwise, we have to remove the top most folder.
            // So e.g. /foo/bar/baz.md => /foo/bar/baz => /foo/bar
            //              ^                   ^             ^
            //              |                   |             |
            //          fs_path               as URI      parent folder URI
            let mut parts = uri.split('/').collect::<Vec<_>>();
            parts.pop();
            parts.join("/")
        };

        self.relative_url_base = Some(base);
    }

    pub fn with_url_base_by_page_uri(&mut self, page_uri: &str) {
        if let Ok(mut path) = uriparse::Path::try_from(page_uri) {
            path.pop();

            self.relative_url_base = Some(path.to_string());
        } else {
            panic!("Page URI was not valid {}", page_uri);
        }
    }

    pub fn should_expand_relative_uris(&self) -> bool {
        self.relative_url_base.is_some()
    }

    #[cfg(test)]
    pub fn with_cache_bust_timestamp(&mut self, timestamp: String) {
        self.cache_bust_timestamp = timestamp;
    }
}

impl std::fmt::Debug for RenderContext<'_> {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("RenderContext")
            .field("options", &self.options)
            .field("settings", &self.settings)
            .field("relative_url_base", &self.relative_url_base)
            .field("custom_components", &self.custom_components)
            .field("assets", &self.assets)
            .finish()?;

        Ok(())
    }
}

/// The context of the current file that is being rendered.
#[derive(Clone, Debug)]
pub struct FileContext {
    pub fs_path: PathBuf,
    pub error_lines_offset: usize,
    pub error_bytes_offset: usize,
}

impl FileContext {
    pub fn new(error_lines_offset: usize, error_bytes_offset: usize, fs_path: PathBuf) -> Self {
        FileContext {
            error_lines_offset,
            error_bytes_offset,
            fs_path,
        }
    }
}
