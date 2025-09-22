use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
/// Rules for rendering a Doctave page, for rewriting links,
/// prefixing asset URLs, settings user preferences, etc.
///
/// Note that bool fields are `false` by default according to
/// Rust's default rules
#[derive(Default)]
pub struct RenderOptions {
    pub bust_image_caches: bool,
    /// Convert any .md links to their web equivalent
    ///
    /// NOTE:: Cannot be used with `fsify_internal_urls`
    pub webbify_internal_urls: bool,
    /// Internal URLs are rewritten as filesystem paths in the project.
    /// E.g. /foo/bar becomes /foo/bar.md (or /foo/bar/README.md)
    ///
    /// NOTE:: Cannot be used with `webbify_internal_urls`
    pub fsify_internal_urls: bool,
    pub disable_syntax_highlighting: bool,
    pub link_rewrites: HashMap<String, String>,
    pub prefix_asset_urls: Option<String>,
    pub prefix_link_urls: Option<String>,
    pub user_preferences: HashMap<String, String>,
    pub download_url_prefix: Option<String>,
}
