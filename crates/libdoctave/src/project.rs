use include_dir::{include_dir, Dir, DirEntry};
use itermap::IterMap;
use rayon::prelude::*;
use std::collections::hash_map::DefaultHasher;
use std::ffi::OsStr;
use std::hash::Hasher;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::content_api::{ContentApiResponse, ResponseContext};
use crate::error_options::ErrorOptions;
use crate::open_api::ast::PageAst;
use crate::open_api::model::Components;
use crate::open_api::OpenApi;
use crate::page_handle::PageHandle;
use crate::page_kind::PageKind;
use crate::render_context::{FileContext, RenderContext};
use crate::settings::Settings;
use crate::tabs::TabsList;
use crate::SearchIndex;

use crate::vale::{vale_results_to_errors, vale_runtime_error_to_error};
use crate::{
    ast_mdx_fault_tolerant, frontmatter, markdown_navigation, navigation, renderable_ast,
    uri_to_fs_path, Ast, CustomComponentHandle, Error, MarkdownPage, RenderOptions,
    BAKED_COMPONENTS, DEPRECATED_NAVIGATION_FILE_NAME, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
};
use std::collections::HashMap;

static BOILERPLATE_PROJECT: Dir = include_dir!("./crates/libdoctave/boilerplate_project");

#[derive(Clone, Debug)]
pub(crate) enum NavigationHandle {
    LegacyMarkdown(String),
    Yaml(String),
}

impl NavigationHandle {
    fn file_name(&self) -> String {
        match &self {
            NavigationHandle::LegacyMarkdown(_) => DEPRECATED_NAVIGATION_FILE_NAME.to_string(),
            NavigationHandle::Yaml(_) => NAVIGATION_FILE_NAME.to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Asset {
    pub path: PathBuf,
    pub signature: u64,
}

impl PartialEq for Asset {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl PartialEq<PathBuf> for Asset {
    fn eq(&self, other: &PathBuf) -> bool {
        &self.path == other
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct PageOptions {
    pub hide_navigation: bool,
    pub hide_side_table_of_contents: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
/// A beter typed interface for ensuring that we are explicit about the kinds of
/// files we get.
pub struct InputFile {
    pub path: PathBuf,
    pub content: InputContent,
}

impl InputFile {
    fn into_internal_repr(self) -> (PathBuf, String) {
        match self.content {
            InputContent::Binary(signature) => (self.path, signature),
            InputContent::Text(t) => (self.path, t),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum InputContent {
    Binary(String),
    Text(String),
}

impl InputContent {
    pub fn text(&self) -> Option<&str> {
        match &self {
            InputContent::Binary(_) => None,
            InputContent::Text(t) => Some(t.as_str()),
        }
    }
}

#[derive(Clone)]
pub struct Project {
    navigations: Option<HashMap<String, Option<NavigationHandle>>>,
    pub(crate) pages: Vec<PageKind>,
    tabs: Option<TabsList>,
    /// Number of bytes taken by all the content in this project.
    /// Does **not** include size of assets.
    pub content_size_bytes: usize,
    pub assets: Vec<Asset>,
    pub settings: Arc<Settings>,
    /// After parsing we loose information about what files
    /// were actually available, but want to still be able to
    /// report errors in the `verify` step. So we'll keep
    /// a list of input paths, without contents, in memory for now.
    pub(crate) input_paths: Vec<PathBuf>,
    pub(crate) custom_components: Vec<CustomComponentHandle>,
    pub(crate) open_api_components: HashMap<String, Components>,
    pub custom_css: Vec<String>,
}

impl Project {
    /// Construct a project from a file list. This is the primary way to create a
    /// project since we cannot rely on doing IO in libdoctave.
    ///
    /// Expects a list of tuples of PathBufs and Strings that represent the path
    /// and content of the file.
    ///
    /// Note that files under the _asset folder should also be passed in, but the
    /// content of the files can be empty, since the contents of assets is not
    /// verified in any way.
    ///
    pub fn from_file_list(list: Vec<InputFile>) -> Result<Project, Vec<Error>> {
        let input_paths = list.iter().map(|i| i.path.clone()).collect::<Vec<_>>();

        let content_size_bytes =
            list.iter()
                .filter_map(|c| c.content.text())
                .fold(0, |mut acc, t| {
                    acc += t.bytes().len();
                    acc
                });

        let list = list
            .into_iter()
            .map(|f| f.into_internal_repr())
            .collect::<Vec<_>>();

        let settings: Arc<Settings> = {
            if let Some((_path, content)) = list
                .iter()
                .find(|(path, _)| path == Path::new(SETTINGS_FILE_NAME))
            {
                Settings::parse(content)
            } else {
                Err(Error {
                    code: Error::MISSING_DOCTAVE_YAML,
                    message: "Missing docapella.yaml".to_string(),
                    description: "No docapella.yaml file found in the root of your project."
                        .to_owned(),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                })
            }
        }
        .map(Arc::new)
        .map_err(|e| vec![e])?;

        let mut assets = Vec::new();
        let mut pages = Vec::new();
        let mut custom_components = BAKED_COMPONENTS.to_vec();
        let mut open_api_components = HashMap::new();

        // Go through all files in the list, sorting out partials and pages
        for (path, content) in list
            .iter()
            .filter(|(path, _)| path != Path::new(DEPRECATED_NAVIGATION_FILE_NAME))
            .filter(|(path, _)| path != Path::new(SETTINGS_FILE_NAME))
        {
            if path.starts_with("_components") || path.starts_with("_topics") {
                custom_components.push(CustomComponentHandle::new(content, path));
            }

            if path.starts_with("_assets") {
                let mut hasher = DefaultHasher::new();

                hasher.write(content.as_bytes());

                assets.push(Asset {
                    path: path.clone(),
                    signature: hasher.finish(),
                });
            }
        }

        // Gather open_api specs
        for spec in settings.open_api() {
            let mut hasher = DefaultHasher::new();

            hasher.write(spec.spec_file.to_string_lossy().to_string().as_bytes());

            assets.push(Asset {
                path: spec.spec_file.clone(),
                signature: hasher.finish(),
            });

            if let Some(entry) = list.iter().find(|(p, _)| p == &spec.spec_file) {
                let mut parsed_spec = Self::parse_openapi_spec(spec, &entry.1)?;

                let openapi_pages = OpenApi::pages_from_parsed_spec(
                    &parsed_spec,
                    spec.spec_file.clone(),
                    spec.uri_prefix.clone(),
                )
                .map_err(|e| vec![e])?;

                for page in openapi_pages {
                    pages.push(page);
                }

                open_api_components.insert(
                    spec.spec_file.to_string_lossy().to_string(),
                    OpenApi::components_parsed(&mut parsed_spec).map_err(|e| vec![e])?,
                );
            } else {
                // Skip - we tried searching for an OpenAPI spec, but couldn't find one.
                // The verify step will detect this later, and show an error to the user.
            }
        }

        let tabs = settings.tabs();

        let navigations: Option<HashMap<_, _>> = if let Some(ref tabs) = tabs {
            let mut navs = vec![];
            let paths = tabs.nav_paths();

            paths.iter().for_each(|path| {
                let nav_file_path = PathBuf::from(
                    format!("{}/{}", path, NAVIGATION_FILE_NAME).trim_start_matches('/'),
                );

                let handle = list
                    .iter()
                    .find(|(file_path, _)| file_path == &nav_file_path)
                    .map(|(_, c)| NavigationHandle::Yaml(c.clone()));

                navs.push((path.to_string(), handle));
            });

            if navs.is_empty() {
                None
            } else {
                Some(HashMap::from_iter(navs))
            }
        } else {
            let navigation_handle = match (
                list.iter()
                    .find(|(p, _)| p == Path::new(DEPRECATED_NAVIGATION_FILE_NAME)),
                list.iter()
                    .find(|(p, _)| p == Path::new(NAVIGATION_FILE_NAME)),
            ) {
                (None, Some((_, content))) => Some(NavigationHandle::Yaml(content.clone())),
                (Some(_), Some((_, content))) => Some(NavigationHandle::Yaml(content.clone())),
                (Some((_, old)), None) => Some(NavigationHandle::LegacyMarkdown(old.clone())),
                (None, None) => None,
            };

            navigation_handle.map(|nav| HashMap::from_iter(vec![("/".to_string(), Some(nav))]))
        };

        for (path, content) in list
            .iter()
            .filter(|(path, _)| path != Path::new(DEPRECATED_NAVIGATION_FILE_NAME))
            .filter(|(path, _)| path != Path::new(SETTINGS_FILE_NAME))
            .filter(|(path, _)| path.extension() == Some(std::ffi::OsStr::new("md")))
        {
            if !path.starts_with("_partials")
                && !path.starts_with("_components")
                && !path.starts_with("_topics")
            {
                pages.push(PageKind::Markdown(MarkdownPage::new(
                    path,
                    content.as_bytes().to_owned(),
                )));
            }
        }

        let custom_css = settings
            .styles()
            .iter()
            .flat_map(|path| list.iter().find(|(p, _)| p == path).map(|(_, c)| c.clone()))
            .collect::<Vec<_>>();

        // Safe to unwrap here as errors have been found already
        Ok(Project {
            navigations,
            tabs,
            content_size_bytes,
            settings,
            custom_css,
            pages,
            assets,
            input_paths,
            custom_components,
            open_api_components,
        })
    }

    pub fn parse_openapi_spec(
        spec: &crate::settings::OpenApi,
        content: &str,
    ) -> Result<openapi_parser::OpenAPI, Vec<Error>> {
        match spec.spec_file.extension().and_then(OsStr::to_str) {
            Some("json") => openapi_parser::openapi30::parser::parse_json(content).map_err(|e| {
                vec![Error {
                    code: Error::INVALID_OPENAPI_SPEC,
                    message: "Could not parse OpenAPI spec".to_owned(),
                    description: e.to_string(),
                    file: Some(spec.spec_file.clone()),
                    position: None,
                }]
            }),
            Some("yaml") => openapi_parser::openapi30::parser::parse_yaml(content).map_err(|e| {
                vec![Error {
                    code: Error::INVALID_OPENAPI_SPEC,
                    message: "Could not parse OpenAPI spec".to_owned(),
                    description: e.to_string(),
                    file: Some(spec.spec_file.clone()),
                    position: None,
                }]
            }),
            _ => Err(vec![Error {
                code: Error::INVALID_OPENAPI_SPEC,
                message: "Could not parse OpenAPI spec".to_owned(),
                description: "OpenAPI spec must be a JSON or YAML file.".to_string(),
                file: Some(spec.spec_file.clone()),
                position: None,
            }])?,
        }
    }

    pub fn openapi_ast_for_tag(
        &self,
        spec: &crate::settings::OpenApi,
        content: &str,
        tag: Option<&str>,
        opts: Option<&RenderOptions>,
    ) -> Result<(PageAst, Vec<String>), Vec<Error>> {
        let parsed_spec = Self::parse_openapi_spec(spec, content)?;

        let all_tags = parsed_spec.tag_names();

        if let Ok(openapi_pages) = OpenApi::pages_from_parsed_spec(
            &parsed_spec,
            spec.spec_file.clone(),
            spec.uri_prefix.clone(),
        ) {
            if let Some(oapi) = openapi_pages.iter().find(|p| p.openapi_tag() == tag) {
                let mut ctx = RenderContext::new();
                ctx.with_maybe_options(opts);
                ctx.with_project(self);

                return match oapi.ast(&mut ctx) {
                    Ok(Ast::OpenApi(oapi)) => Ok((oapi, all_tags)),
                    Err(e) => Err(vec![e]),
                    _ => Err(vec![]),
                };
            }
        }

        Err(vec![])
    }

    pub fn openapi_overview(
        &self,
        spec: &crate::settings::OpenApi,
        content: &str,
        opts: Option<&RenderOptions>,
    ) -> Result<(renderable_ast::Node, Vec<String>), Vec<Error>> {
        let parsed_spec = Self::parse_openapi_spec(spec, content)?;

        let all_tags = parsed_spec.tag_names();

        if let Ok(openapi_pages) = OpenApi::pages_from_parsed_spec(
            &parsed_spec,
            spec.spec_file.clone(),
            spec.uri_prefix.clone(),
        ) {
            if let Some(overview_page) = openapi_pages.iter().find(|p| p.markdown().is_some()) {
                let mut ctx = RenderContext::new();
                ctx.with_maybe_options(opts);
                ctx.with_project(self);

                return match overview_page.ast(&mut ctx) {
                    Ok(Ast::Markdown(markdown_ast)) => Ok((markdown_ast, all_tags)),
                    Err(e) => Err(vec![e]),
                    _ => Err(vec![]),
                };
            }
        }

        Err(vec![])
    }

    pub fn get_openapi_paths(settings_content: String) -> crate::Result<Vec<String>> {
        let settings = Settings::parse(&settings_content)?;

        Ok(settings
            .open_api()
            .iter()
            .map(|spec| spec.spec_file.to_string_lossy().to_string())
            .collect())
    }

    pub fn get_content_response_by_uri_path(
        &self,
        uri_path: &str,
        ctx: ResponseContext,
    ) -> ContentApiResponse {
        let page = self.get_page_by_uri_path(uri_path);

        if let Some(page) = page {
            ContentApiResponse::content(page, self, ctx)
        } else {
            ContentApiResponse::page_not_found(uri_path, self, ctx)
        }
    }

    pub fn get_content_response_as_json_string_by_uri_path(
        &self,
        uri_path: &str,
        mut ctx: ResponseContext,
    ) -> (String, u16) {
        ctx.debug_info.start_performance("LIBDOCTAVE_CONTENT");
        let page = self.get_page_by_uri_path(uri_path);

        let mut res = if let Some(page) = page {
            ContentApiResponse::content(page, self, ctx)
        } else {
            ContentApiResponse::page_not_found(uri_path, self, ctx)
        };

        if let ContentApiResponse::Content {
            ref mut debug_info, ..
        } = res
        {
            debug_info.end_performance("LIBDOCTAVE_CONTENT");
        };

        (
            serde_json::to_string(&res).expect("JSON serialization failed. WTF?"),
            res.response_status(),
        )
    }

    pub fn get_ast_mdx_fault_tolerant(
        &self,
        markdown: &str,
        fs_path: &Path,
        render_opts: &RenderOptions,
    ) -> std::result::Result<renderable_ast::Node, (Option<renderable_ast::Node>, Vec<Error>)> {
        let mut ctx = RenderContext::new();

        ctx.with_file_context(FileContext::new(
            markdown[..frontmatter::end_pos(markdown)].lines().count(),
            frontmatter::end_pos(markdown),
            fs_path.to_owned(),
        ));

        ctx.with_maybe_options(Some(render_opts));
        ctx.with_project(self);

        ast_mdx_fault_tolerant(frontmatter::without(markdown), &ctx)
    }

    pub fn fs_path_to_uri_path(&self, fs_path: &Path, tag_name: Option<&str>) -> String {
        self.pages()
            .iter()
            .find(|p| match p.page {
                PageKind::Markdown(_) => p.fs_path() == fs_path,
                PageKind::OpenApi(o) => p.fs_path() == fs_path && o.tag() == tag_name,
            })
            .map(|p| p.uri_path().to_owned())
            .unwrap_or_else(|| fs_path.display().to_string())
    }

    pub fn check_features(&self) -> Vec<String> {
        let mut features = vec![];

        if !self.settings.user_preferences().is_empty() {
            features.push("user_preferences".to_string());
        }

        if !self.settings.styles().is_empty() {
            features.push("custom_css".to_string());
        }

        features
    }

    pub fn verify(
        &self,
        opts: Option<&RenderOptions>,
        err_opts: Option<&ErrorOptions>,
    ) -> Result<(), Vec<Error>> {
        let mut errors = vec![];

        if let Some(vale_results) = err_opts.and_then(|o| o.external_results.clone()) {
            for error in vale_results_to_errors(self, vale_results) {
                errors.push(error);
            }
        }

        if let Some(vale_runtime_error) = err_opts.and_then(|o| o.vale_runtime_error.clone()) {
            if let Some(config_file_path) = self
                .settings()
                .vale()
                .and_then(|v| v.config_file_path.as_ref())
            {
                let error = vale_runtime_error_to_error(vale_runtime_error, config_file_path);
                errors.push(error);
            }
        }

        for error in self.verify_root_readme() {
            errors.push(error);
        }

        for error in self.verify_tabs() {
            errors.push(error);
        }

        for error in self.verify_navigation(opts) {
            errors.push(error);
        }

        for error in self.verify_frontmatters() {
            errors.push(error);
        }

        self.settings.verify(self, &mut errors);

        let shared = std::sync::Mutex::new(&mut errors);

        self.pages().iter().for_each(|p| {
            if let Err(error) = p.ast(opts) {
                let mut e = shared.lock().unwrap();
                e.push(error);
            }
        });

        let mut ctx = RenderContext::new();
        ctx.with_project(self);
        ctx.with_maybe_options(opts);

        for handle in &self.custom_components {
            if let Err(errs) = handle.verify(&ctx) {
                for e in errs {
                    errors.push(Error {
                        code: Error::INVALID_COMPONENT,
                        message: e.to_string(),
                        description: e.render(&handle.content, &ctx),
                        file: Some(handle.path.clone()),
                        position: None,
                    });
                }
            }
            if let Err(e) = handle.build() {
                errors.push(Error {
                    code: Error::INVALID_COMPONENT,
                    message: e.to_string(),
                    description: e.render(&handle.content, &ctx),
                    file: Some(handle.path.clone()),
                    position: None,
                })
            }
        }

        self.verify_internal_links(&mut errors);

        if !errors.is_empty() {
            errors.sort();
            errors.dedup();

            Err(errors)
        } else {
            Ok(())
        }
    }

    fn verify_frontmatters(&self) -> Vec<Error> {
        let mut errors = vec![];

        let pages = self.pages();
        let markdown_pages = pages.iter().filter_map(|p| match p.page {
            PageKind::Markdown(md) => Some(md),
            _ => None,
        });

        for page in markdown_pages {
            // First, check that we have a valid frontmatter
            if let Err(error) = page.frontmatter() {
                errors.push(error);
            }
        }

        errors
    }

    fn verify_internal_links(&self, errors: &mut Vec<Error>) {
        self.verify_page_links(errors);
        self.verify_navigation_links(errors);
    }

    fn verify_page_links(&self, mut errors: &mut Vec<Error>) {
        let shared = std::sync::Mutex::new(&mut errors);

        let render_opts = if !self.settings().user_preferences().is_empty() {
            self.settings
                .user_preference_combinations()
                .into_iter()
                .fold(vec![], |mut acc, combos| {
                    let opts = RenderOptions {
                        user_preferences: combos.into_iter().map_values(|val| val.value).collect(),
                        ..Default::default()
                    };
                    acc.push(opts);
                    acc
                })
        } else {
            vec![RenderOptions::default()]
        };

        self.pages().par_iter().for_each(|p| {
            for opts in &render_opts {
                if let Ok(links) = p.outgoing_links(Some(opts)) {
                    for link in links.iter() {
                        let path = PathBuf::from(link.expanded_uri.as_ref().unwrap_or(&link.uri));
                        let uri = crate::fs_to_uri_path(&path);

                        if self.get_page_by_uri_path(&uri).is_none()
                            && !self.redirects().iter().any(|r| r.0 == uri)
                        {
                            let error = if p.is_markdown() {
                                Error {
                                    code: Error::BROKEN_INTERNAL_LINK,
                                    message: String::from("Broken link detected"),
                                    description: format!(
                                        "Link {} points to an unknown file.",
                                        link.uri
                                    ),
                                    file: Some(p.fs_path().to_owned()),
                                    position: None,
                                }
                            } else {
                                Error {
                                    code: Error::BROKEN_INTERNAL_LINK,
                                    message: String::from("Broken link in OpenAPI spec"),
                                    description: format!(
                                        "Link {} in OpenAPI spec {} points to an unknown file.",
                                        link.uri,
                                        p.fs_path().display(),
                                    ),
                                    file: Some(PathBuf::from(p.uri_path())),
                                    position: None,
                                }
                            };
                            let mut e = shared.lock().unwrap();
                            e.push(error);
                        }
                    }
                }

                if let Ok(asset_links) = p.asset_links(Some(opts)) {
                    for link in asset_links.iter() {
                        let path = PathBuf::from(link.expanded_uri.as_ref().unwrap_or(&link.uri));

                        if self.get_asset_by_fs_path(&path).is_none() {
                            let error = Error {
                                code: Error::BROKEN_INTERNAL_LINK,
                                message: String::from("Broken asset link detected"),
                                description: format!(
                                    "Link {} points to an unknown file.",
                                    link.uri
                                ),
                                file: Some(p.fs_path().to_owned()),
                                position: None,
                            };

                            let mut e = shared.lock().unwrap();
                            e.push(error);
                        }
                    }
                }
            }
        });
    }

    fn verify_navigation_links(&self, errors: &mut Vec<Error>) {
        let render_opts = if !self.settings().user_preferences().is_empty() {
            self.settings
                .user_preference_combinations()
                .into_iter()
                .fold(vec![], |mut acc, combos| {
                    let opts = RenderOptions {
                        user_preferences: combos.into_iter().map_values(|val| val.value).collect(),
                        ..Default::default()
                    };
                    acc.push(opts);
                    acc
                })
        } else {
            vec![RenderOptions::default()]
        };

        for opts in &render_opts {
            if let Some(navigations) = &self.navigations {
                let navigations_with_handle = navigations
                    .iter()
                    .filter_map(|(path, handle)| handle.as_ref().map(|h| (path, h)));
                for (subtab_path, nav_handle) in navigations_with_handle {
                    let nav_file_path = PathBuf::from(subtab_path).join(nav_handle.file_name());

                    if let Ok(nav) = self.navigation(Some(opts), subtab_path) {
                        for internal_link in nav.gather_links() {
                            if self
                                .get_page_by_uri_path(&internal_link)
                                .or_else(|| {
                                    self.get_page_by_fs_path(&uri_to_fs_path(&internal_link))
                                })
                                .is_none()
                                && !self.redirects().iter().any(|r| r.0 == internal_link)
                            {
                                let error = Error {
                                    code: Error::BROKEN_INTERNAL_LINK,
                                    message: String::from("Broken link detected in navigation"),
                                    description: format!(
                                        "Link {} points to an unknown file.",
                                        &internal_link
                                    ),
                                    file: Some(nav_file_path.to_owned()),
                                    position: None,
                                };
                                errors.push(error);
                            }
                        }
                    }
                }
            }
        }
    }

    fn verify_root_readme(&self) -> Vec<Error> {
        let mut errors = vec![];

        if let Some(tabs) = self.tabs() {
            for tab in &tabs.tabs {
                if !tab.is_external && self.get_page_by_uri_path(&tab.href).is_none() {
                    errors.push(Error {
                                code: Error::MISSING_ROOT_README,
                                message: format!(r#"Missing root README.md for tab "{}". Add a file at "{}/README.md"."#, tab.label, tab.href),
                                description: "All your project's tabs have to have a root README.md file. This is the first page readers will see in your tab.".to_owned(),
                                file: None,
            position: None,
                            });
                }

                for subtab in &tab.subtabs {
                    if !subtab.is_external && self.get_page_by_uri_path(&subtab.href).is_none() {
                        errors.push(Error {
                                  code: Error::MISSING_ROOT_README,
                                  message: format!(r#"Missing root README.md for subtab "{}". Add a file at "{}/README.md"."#, subtab.label, subtab.href),
                                  description: "All your project's tabs have to have a root README.md file. This is the first page readers will see in your tab.".to_owned(),
                                  file: None,
            position: None,
                              });
                    }
                }
            }
        } else if self.get_page_by_fs_path(Path::new("README.md")).is_none() {
            errors.push(Error {
                code: Error::MISSING_ROOT_README,
                message: r#"Missing root README.md. Add a file at "/README.md"."#.to_owned(),
                description: "Your project has to have a root README.md file. This is the first page readers will see in your project.".to_owned(),
                file: None,
            position: None,
            });
        }

        errors
    }

    fn verify_tabs(&self) -> Vec<Error> {
        match &self.tabs {
            Some(tabs) => tabs.verify(),
            None => vec![],
        }
    }

    /// Verifies that the structure of the navigation is correct.
    /// Note this does not check for broken links.
    fn verify_navigation(&self, opts: Option<&RenderOptions>) -> Vec<Error> {
        let mut errors = vec![];

        if self.navigations.is_none() {
            let message = if self.tabs.is_none() {
                "Missing navigation.yaml in project root".to_owned()
            } else {
                "Missing a navigation.yaml".to_owned()
            };

            errors.push(Error {
                code: Error::MISSING_NAVIGATION,
                message,
                description: "Could not build navigation structure".to_owned(),
                file: None,
                position: None,
            });
        }

        if let Some(navs) = &self.navigations {
            for (subtab_path, nav_handle) in navs {
                let mut errors_for_nav = vec![];
                if let Some(nav_handle) = nav_handle {
                    let nav_file_path = PathBuf::from(subtab_path).join(nav_handle.file_name());

                    match nav_handle {
                        NavigationHandle::Yaml(ref content) => {
                            let mut nav_errors = navigation::verify(content, self);
                            errors_for_nav.append(&mut nav_errors);
                        }
                        NavigationHandle::LegacyMarkdown(_) => {
                            if let Err(e) = self.root_navigation(opts) {
                                errors_for_nav.push(e);
                            }
                        }
                    }

                    // This is kind of ugly, but right now a bit hesitant to pass context
                    // into navigation verification just for error reporting as it runs
                    // quite deep into it. Therefore, we just mutate the file path after
                    // verification has ran.
                    errors_for_nav
                        .iter_mut()
                        .for_each(|error| error.file = Some(nav_file_path.to_owned()));

                    errors.append(&mut errors_for_nav);
                } else {
                    errors.push(Error {
                        code: Error::MISSING_NAVIGATION,
                        message: match subtab_path.as_str() {
                            "/" => "Missing navigation.yaml in project root".to_owned(),
                            _ => "Missing navigation.yaml for tab".to_owned(),
                        },
                        description: format!("Could not find navigation.yaml in `{}`", subtab_path),
                        file: None,
                        position: None,
                    });
                }
            }
        }

        errors
    }

    pub fn get_external_links(&self) -> Vec<String> {
        let render_opts = if !self.settings().user_preferences().is_empty() {
            self.settings
                .user_preference_combinations()
                .into_iter()
                .fold(vec![], |mut acc, combos| {
                    let opts = RenderOptions {
                        user_preferences: combos.into_iter().map_values(|val| val.value).collect(),
                        ..Default::default()
                    };
                    acc.push(opts);
                    acc
                })
        } else {
            vec![RenderOptions::default()]
        };

        self.pages()
            .iter()
            .flat_map(|p| {
                let mut out = vec![];

                for opts in &render_opts {
                    if let Ok(links) = p.external_links(Some(opts)) {
                        out.append(&mut links.to_vec());
                    }
                }

                out
            })
            .collect()
    }

    pub fn search_index(&self) -> crate::Result<SearchIndex> {
        SearchIndex::new(self)
    }

    pub fn boilerplate_file_list() -> Vec<(PathBuf, Vec<u8>)> {
        let mut files = vec![];

        fn gather_recursively(entries: &[DirEntry], files: &mut Vec<(PathBuf, Vec<u8>)>) {
            for entry in entries {
                match entry {
                    DirEntry::Dir(d) => gather_recursively(d.entries(), files),
                    DirEntry::File(file) => {
                        // Make sure we don't dump .DS_Store files
                        if !file.path().ends_with(".DS_Store") {
                            files.push((file.path().to_owned(), file.contents().to_owned()));
                        }
                    }
                }
            }
        }

        gather_recursively(BOILERPLATE_PROJECT.entries(), &mut files);

        files
    }

    pub fn root_navigation(
        &self,
        opts: Option<&RenderOptions>,
    ) -> crate::Result<navigation::Navigation> {
        self.navigation(opts, "/")
    }

    pub fn navigation(
        &self,
        opts: Option<&RenderOptions>,
        subtab_path: &str,
    ) -> crate::Result<navigation::Navigation> {
        let mut ctx = RenderContext::new();
        ctx.with_project(self);
        ctx.with_maybe_options(opts);

        let mut path = subtab_path.to_string();

        if let Some(_structure) = &self.tabs {
            path = format!(
                "/{}",
                subtab_path.trim_start_matches('/').trim_end_matches('/')
            )
        }

        match path.as_str() {
            "/" => {}
            path => {
                ctx.with_url_base(path);
            }
        };

        match &self.navigations {
            Some(navs) => match navs.get(&path) {
                Some(Some(NavigationHandle::LegacyMarkdown(ref md))) => {
                    markdown_navigation::Navigation::from_markdown(md, &ctx)
                }
                Some(Some(NavigationHandle::Yaml(ref yaml))) => navigation::build(yaml, &ctx, self),
                None | Some(None) => Err(Error {
                    code: Error::MISSING_NAVIGATION,
                    message: "Missing navigation.yaml for tab".to_owned(),
                    description: format!("Could not find navigation.yaml in `{}`", subtab_path),
                    file: None,
                    position: None,
                }),
            },
            None => Err(Error {
                code: Error::MISSING_NAVIGATION,
                message: "Missing navigation.yaml in project root".to_owned(),
                description: "Could not build navigation structure".to_owned(),
                file: None,
                position: None,
            }),
        }
    }

    pub fn navigation_has_link_to(&self, path: &str, opts: Option<&RenderOptions>) -> bool {
        self.navigations
            .as_ref()
            .map(|navs| {
                navs.keys().any(|subtab_path| {
                    if let Ok(nav) = self.navigation(opts, subtab_path) {
                        nav.has_link_to(path)
                    } else {
                        false
                    }
                })
            })
            .unwrap_or(false)
    }

    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    pub fn tabs(&self) -> Option<&TabsList> {
        self.tabs.as_ref()
    }

    pub fn autocomplete(
        &self,
        markdown: &str,
        fs_path: &Path,
        render_opts: Option<&RenderOptions>,
    ) -> Vec<crate::CompletionItem> {
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(render_opts);
        ctx.with_project(self);

        crate::markdown::autocomplete(markdown, fs_path, self, &ctx)
    }

    pub fn get_page_by_uri_path(&self, uri_path: &str) -> Option<PageHandle> {
        // If we get an anchor in the URI, remove it.
        let without_anchor = uri_path.split('#').collect::<Vec<_>>()[0];

        for page in &self.pages {
            if page.uri_path() == without_anchor {
                return Some(PageHandle {
                    page,
                    project: self,
                });
            }
        }

        None
    }

    pub fn get_page_by_fs_path(&self, path: &Path) -> Option<PageHandle> {
        for page in &self.pages {
            if let PageKind::Markdown(md) = page {
                if Self::normalize_fs_path(md.path.as_ref()) == Self::normalize_fs_path(path) {
                    return Some(PageHandle {
                        page,
                        project: self,
                    });
                }
            }
        }

        None
    }

    pub fn get_asset_by_fs_path(&self, path: &Path) -> Option<&Asset> {
        self.assets
            .iter()
            .find(|a| Self::normalize_fs_path(&a.path) == Self::normalize_fs_path(path))
    }

    pub fn get_subtab_path_by_uri_path(&self, uri_path: &str) -> Option<String> {
        match &self.tabs {
            Some(tabs) => {
                let normalized_uri_path = format!(
                    "/{}",
                    uri_path.trim_end_matches('/').trim_start_matches('/')
                );

                let mut matching_paths = vec!["/".to_string()];

                for tab in tabs.tabs.iter() {
                    if normalized_uri_path.contains(&tab.href) {
                        matching_paths.push(tab.href.clone());
                    }

                    for subtab in &tab.subtabs {
                        if normalized_uri_path.contains(&subtab.href) {
                            matching_paths.push(subtab.href.clone());
                        };
                    }
                }

                /*
                    We get the deepest match. For examples, /foo/bar/md
                    would return /foo/bar, not /foo.
                */
                matching_paths
                    .iter()
                    .enumerate()
                    .max_by_key(|(_, path)| path.len())
                    .map(|(_, value)| value)
                    .cloned()
            }
            None => None,
        }
    }

    fn normalize_fs_path(path: &Path) -> &Path {
        path.strip_prefix("/").unwrap_or(path)
    }

    pub fn pages(&self) -> Vec<PageHandle> {
        self.pages
            .iter()
            .map(|page| PageHandle {
                page,
                project: self,
            })
            .collect::<Vec<_>>()
    }

    fn settings_redirects(&self) -> Vec<(String, String)> {
        self.settings
            .redirects()
            .iter()
            .map(|r| r.as_tuple())
            .collect()
    }

    pub fn redirects(&self) -> Vec<(String, String)> {
        let mut redirects = vec![];

        redirects.append(&mut self.settings_redirects());

        redirects
    }
}

impl std::fmt::Debug for Project {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        fmt.debug_struct("Project")
            .field("navigation", &self.root_navigation(None))
            .field("pages", &self.pages)
            .field("settings", &self.settings)
            .finish()?;

        Ok(())
    }
}

impl Default for Project {
    fn default() -> Self {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
              items:
                - href: foo/bar.md
                  label: Example
            "#})),
            },
        ];

        Self::from_file_list(file_list).unwrap()
    }
}

#[cfg(test)]
mod test {
    use crate::Ast;
    use pretty_assertions::assert_str_eq;

    use super::*;

    #[test]
    fn verifies_each_page() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project\n")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("{% badliquid, %}".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        assert!(project.verify(Some(&opts), None).is_err());
    }

    #[test]
    fn verifies_missing_root_navigation() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(error.message, "Missing navigation.yaml in project root");
        assert_eq!(error.description, "Could not build navigation structure");
    }

    #[test]
    fn verifies_custom_components() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project\n")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    <Component.Example />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("_components/example.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    <Foo></Bar>
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        assert!(
            project.verify(Some(&opts), None).is_err(),
            "Component not verified"
        );
    }

    #[test]
    fn custom_components_topic_alias() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    <Topic.Example />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("_topics/example.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    # This is a nice topic
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        if let Ast::Markdown(root) = ast {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <Heading1>
                    <Text>
                        This is a nice topic
                    </Text>
                </Heading1>
                "#}
                .to_string()
            );
        } else {
            panic!("Expected markdown AST");
        }
    }

    #[test]
    fn verifies_custom_components_with_topics_alias() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project\n")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    <Topic.Example />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("_topics/example.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    <Foo></Bar>
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        assert!(
            project.verify(Some(&opts), None).is_err(),
            "Component not verified"
        );
    }

    #[test]
    fn custom_components_with_topics_alias_attributes_work() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <Topic.Example heading="Hello">
                        from topics!
                      </Topic.Example>
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("_topics/example.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    attributes:
                      - title: heading
                        required: true
                    ---

                    **{ @heading }**

                    <Slot />
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        if let Ast::Markdown(root) = ast {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Strong>
                        <Text>
                            Hello
                        </Text>
                    </Strong>
                </Paragraph>
                <Paragraph>
                    <Text>
                        from topics!
                    </Text>
                </Paragraph>
                "#}
                .to_string()
            );
        } else {
            panic!("Expected markdown AST");
        }
    }

    #[test]
    fn verifies_navigation_structures_refer_to_existing_user_preferences() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                  show_if:
                    user_preferences:
                      dont_exist:
                        equals: Foo
                  items:
                    - subheading: Else
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                user_preferences:
                  game:
                    label: Game
                    default: Football
                    values:
                      - Baseball
                      - Football
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Unknown user preference \"dont_exist\" found in navigation"
        );
        assert_eq!(
            error.description,
            "Expected one of [\"game\"].\nFound \"dont_exist\"."
        );
    }

    #[test]
    fn verifies_navigation_structures_equals_matcher_refers_to_existing_value() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                  show_if:
                    user_preferences:
                      game:
                        equals: Water Polo
                  items:
                    - subheading: Else
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                user_preferences:
                  game:
                    label: Game
                    default: Football
                    values:
                      - Baseball
                      - Football
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Unknown value \"Water Polo\" for user preference \"game\" found in navigation"
        );
        assert_eq!(
            error.description,
            "Expected one of [\"Baseball\", \"Football\"].\nFound \"Water Polo\"."
        );
    }

    #[test]
    fn verifies_navigation_structures_one_of_matcher_refers_to_existing_values() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                  show_if:
                    user_preferences:
                      game:
                        one_of:
                          - Water Polo
                          - Baseball
                  items:
                    - subheading: Else
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                user_preferences:
                  game:
                    label: Game
                    default: Football
                    values:
                      - Baseball
                      - Football
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Unknown value \"Water Polo\" for user preference \"game\" found in navigation"
        );
        assert_eq!(
            error.description,
            "Expected any of [\"Baseball\", \"Football\"].\nFound \"Water Polo\"."
        );
    }

    #[test]
    fn verifies_navigation_structures_refer_to_existing_user_preferences_when_no_preferences_set() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                  show_if:
                    user_preferences:
                      dont_exist:
                        equals: Foo
                  items:
                    - subheading: Else
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: Foo")),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let error = &project.verify(Some(&opts), None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Unknown user preference \"dont_exist\" found in navigation"
        );
        assert_eq!(
            error.description,
            "No custom user preferences defined in docapella.yaml."
        );
    }

    #[test]
    fn verifies_the_existence_of_stylesheets_mentioned_in_settings() {
        let files = vec![
            InputFile {
                path: PathBuf::from("_assets/cat.jpg"),
                content: InputContent::Text("".to_string()),
            },
            InputFile {
                path: PathBuf::from("_assets/stylez.css"),
                content: InputContent::Text("".to_string()),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                styles:
                  - _assets/style.css
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Could not find style sheet file at \"_assets/style.css\"."
        );
        assert_eq!(
            error.description,
            "Found [\"_assets/stylez.css\"].\nMake sure the file name is correct and located under the \"_assets\" directory."
        );
    }

    #[test]
    fn verifies_the_existence_of_logo_mentioned_in_settings_v2() {
        let files = vec![
            InputFile {
                path: PathBuf::from("_assets/cat.jpg"),
                content: InputContent::Text("".to_string()),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                theme:
                  logo:
                    src: _assets/logo.png
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Could not find logo at \"_assets/logo.png\"."
        );
        assert_eq!(
            error.description,
            "Found following images: [\"_assets/cat.jpg\"].\nMake sure the file name is correct and located under the \"_assets\" directory."
        );
    }

    #[test]
    fn verifies_the_existence_of_dark_mode_logo_mentioned_in_settings_v2() {
        let files = vec![
            InputFile {
                path: PathBuf::from("_assets/cat.jpg"),
                content: InputContent::Text("".to_string()),
            },
            InputFile {
                path: PathBuf::from("_assets/logo.png"),
                content: InputContent::Text("".to_string()),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                theme:
                  logo:
                    src: _assets/logo.png
                    src_dark: _assets/dark-logo.png
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Could not find dark mode logo at \"_assets/dark-logo.png\"."
        );
        assert_eq!(
            error.description,
            "Found following images: [\"_assets/cat.jpg\", \"_assets/logo.png\"].\nMake sure the file name is correct and located under the \"_assets\" directory."
        );
    }

    #[test]
    fn verifies_the_existence_of_favicon_mentioned_in_settings_v2() {
        let files = vec![
            InputFile {
                path: PathBuf::from("_assets/favicon.ico"),
                content: InputContent::Text("".to_string()),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                theme:
                  favicon:
                    src: _assets/favicon.png
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(
            error.message,
            "Could not find favicon at \"_assets/favicon.png\"."
        );
        assert_eq!(
            error.description,
            "Found following possible favicons: [\"_assets/favicon.ico\"].\nMake sure the file name is correct and located under the \"_assets\" directory."
        );
    }

    #[test]
    fn verifies_the_existence_of_openapi_spec() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                open_api:
                  - spec_file: openapi.json
                    uri_prefix: /api
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let error: &Error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(error.message, "Could not find OpenAPI spec.");
        assert_eq!(
            error.description,
            "OpenAPI spec at \"openapi.json\" not found. Is it in the correct location?"
        );
        assert_eq!(error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_uri_prefix() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r#"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        }
                      }
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                open_api:
                  - spec_file: openapi.json
                    uri_prefix:
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let error: &Error = &project.verify(None, None).unwrap_err()[0];

        assert_eq!(error.message, "OpenAPI URI prefix should contain a path.");
        assert_eq!(
            error.description,
            "Define a uri_prefix for the OpenAPI spec \"openapi.json\" in docapella.yaml. For example, uri_prefix: /api."
        );
        assert_eq!(error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn parses_openapi_schemas() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r#"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        },
                        "components": {
                          "schemas": {
                            "Error": {
                              "type": "object",
                              "description": "Error response",
                              "properties": {
                                "code": {
                                  "type": "integer",
                                  "format": "int32"
                                },
                                "message": {
                                  "type": "string"
                                }
                              }
                            }
                          }
                        }
                      }
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                open_api:
                  - spec_file: openapi.json
                    uri_prefix: /api
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();

        let components = project.open_api_components.get("openapi.json").unwrap();

        assert!(components.schemas.contains_key("Error"));
        assert_eq!(
            components
                .schemas
                .get("Error")
                .as_ref()
                .unwrap()
                .description
                .as_ref()
                .unwrap(),
            "Error response"
        );
    }

    #[test]
    fn verifies_for_overlapping_redirects() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /bar
                    to: /
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(
            settings_error.message,
            "Redirect overlaps with existing page"
        );
        assert_eq!(
            settings_error.description,
            r#"Redirect source "/bar" already exists as a page. Delete or rename the page, or change the redirect source."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_for_broken_redirect() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz
                    to: /foo
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(settings_error.message, "Broken redirect detected");
        assert_eq!(
            settings_error.description,
            r#"Redirect destination "/foo" does not exist."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_for_redirect_handles_anchor() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz
                    to: /bar#anchor
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        assert!(
            &project.verify(None, None).is_ok(),
            "redirect verification with anchor failed"
        );
    }

    #[test]
    fn verifies_for_invalid_redirects_slash() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: baz
                    to: foo
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid redirect detected"
                    && e.description == r#"Redirect source "baz" must start with a forward slash."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "redirect leading / verification failed"
        );

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid redirect detected"
                    && e.description
                        == r#"Redirect destination "foo" must start with a forward slash, or be an external URL."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "redirect leading / verification failed"
        );
    }

    #[test]
    fn doesnt_verify_external_urls() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz
                    to: https://example.com
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        assert!(&project.verify(None, None).is_ok());
    }

    #[test]
    fn verifies_for_invalid_redirects_md() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz.md
                    to: /foo.md
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid redirect detected"
                    && e.description == r#"Redirect source "/baz.md" must not end with .md."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "redirect trailing .md verification failed"
        );

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid redirect detected"
                    && e.description == r#"Redirect destination "/foo.md" must not end with .md."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "redirect trailing .md verification failed"
        );
    }

    #[test]
    fn verifies_for_mismatch_wildcard_redirect() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz
                    to: /foo/:path
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(settings_error.message, "Invalid redirect source");
        assert_eq!(
            settings_error.description,
            r#"Redirect source "/baz" should include a wildcard when `to` has path parameters."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_for_path_param_in_source() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz/:path
                    to: /bar
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(settings_error.message, "Invalid redirect source");
        assert_eq!(
            settings_error.description,
            r#"Redirect source "/baz/:path" can't include path parameters."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_for_too_many_wildcards() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz/*/**
                    to: /bar
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(settings_error.message, "Invalid redirect source");
        assert_eq!(
            settings_error.description,
            r#"Redirect source "/baz/*/**" with a wildcard should end with `.../*` or `.../**`."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_for_not_ending_in_wilcard() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz/**/baz
                    to: /bar
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(settings_error.message, "Invalid redirect source");
        assert_eq!(
            settings_error.description,
            r#"Redirect source "/baz/**/baz" with a wildcard should end with `.../*` or `.../**`."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_redirect_path_param_has_non_alphanumerical() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz/**
                    to: /bar/:pat.h/foo
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();
        let settings_error: &Error = &errors[0];

        assert_eq!(settings_error.message, "Invalid redirect destination");
        assert_eq!(
            settings_error.description,
            r#"Redirect destination "/bar/:pat.h/foo" path parameters can only contain alphanumerics and underscores."#
        );
        assert_eq!(settings_error.file, Some(PathBuf::from("docapella.yaml")));
    }

    #[test]
    fn verifies_redirect_wildcard_success() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Bar redirect overlap
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /**
                    to: /bar
                  - from: /foo/**
                    to: /bar
                  - from: /**
                    to: /bar/:path
                  - from: /**
                    to: /bar/:path/foo

                  - from: /*
                    to: /bar
                  - from: /foo/*
                    to: /bar
                  - from: /*
                    to: /bar/:path
                  - from: /*
                    to: /bar/:path/foo
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();

        assert!(project.verify(None, None).is_ok());
    }

    #[test]
    fn verifies_for_non_existing_internal_header_link() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                header:
                  links:
                    - label: "Doctave"
                      href: "/doctave"
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Broken link detected"
                    && e.description == r#"Header link "/doctave" points to an unknown file."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "header internal link verification failed"
        );
    }

    #[test]
    fn verifies_for_non_existing_internal_footer_link() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                footer:
                  links:
                    - label: "Doctave"
                      href: "/doctave"
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Broken link detected"
                    && e.description == r#"Footer link "/doctave" points to an unknown file."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "footer internal link verification failed"
        );
    }

    #[test]
    fn verifies_for_non_external_header_link() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                header:
                  links:
                    - label: "Doctave"
                      external: "/doctave"
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid header link in found in docapella.yaml"
                    && e.description == r#"Found "/doctave", which is not an external link."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "header external link verification failed"
        );
    }

    #[test]
    fn verifies_for_non_external_footer_link() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                footer:
                  links:
                    - label: "Doctave"
                      external: "/doctave"
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid footer link in found in docapella.yaml"
                    && e.description == r#"Found "/doctave", which is not an external link."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "footer external link verification failed"
        );
    }

    #[test]
    fn verifies_for_non_existing_asset_download_header_link() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                header:
                  links:
                    - label: "Doctave"
                      href: "_assets/doctave"
                      download: true
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Broken asset link detected"
                    && e.description
                        == r#"Header link "_assets/doctave" points to an unknown file."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "header download link verification failed"
        );
    }

    #[test]
    fn verifies_for_non_existing_asset_download_footer_link() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                footer:
                  links:
                    - label: "Doctave"
                      href: "_assets/doctave"
                      download: true
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Broken asset link detected"
                    && e.description
                        == r#"Footer link "_assets/doctave" points to an unknown file."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "footer download link verification failed"
        );
    }

    #[test]
    fn verifies_for_identical_source_destination() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                redirects:
                  - from: /baz
                    to: /baz
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| {
                e.message == "Invalid redirect detected"
                    && e.description
                        == r#"Redirect source "/baz" and destination "/baz" must be different."#
                    && e.file == Some(PathBuf::from("docapella.yaml"))
            }),
            "redirect identical source and destination verification failed"
        );
    }

    #[test]
    fn returns_an_error_if_the_openapi_spec_could_not_be_parsed() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text("Totally not an openapi spec".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                open_api:
                  - spec_file: openapi.json
                    uri_prefix: /api
                "#}
                    .to_string(),
                ),
            },
        ];

        let error = &Project::from_file_list(files).unwrap_err()[0];

        assert_eq!(error.message, "Could not parse OpenAPI spec");
        assert_eq!(error.file, Some(PathBuf::from("openapi.json")));
    }

    #[test]
    fn convert_uri_to_subtab_path_resolves_subtab_path_without_default() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("- heading: Something\n".to_string()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something

                tabs:
                  - label: Tab1
                    path: /tab1/section1/
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        assert_eq!(
            Project::get_subtab_path_by_uri_path(&project, ""),
            Some("/".to_string())
        );
        assert_eq!(
            Project::get_subtab_path_by_uri_path(&project, "/"),
            Some("/".to_string())
        );
    }

    #[test]
    fn verifies_all_navigations() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from("tab1/section1/README.md"),
                content: InputContent::Text("# Hi, this is section 1".to_string()),
            },
            InputFile {
                path: ["tab1/section1/", NAVIGATION_FILE_NAME].iter().collect(),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                  items:
                    - label: Section 1
                      path: /tab1/section1
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                  items:
                    - label: Section 1
                      href: /something/notexists/
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something

                tabs:
                  - label: Default
                    path: /
                  - label: Tab1
                    path: /tab1/section1/
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files.clone()).unwrap();
        let errors: &Vec<Error> = &project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Invalid navigation.yaml");
        assert_eq!(
            errors[0].description,
            ".[0].items: data did not match any variant of untagged enum ItemDescription at line 4 column 5"
        );
        assert_eq!(
            errors[0].file,
            Some(PathBuf::from("/tab1/section1/navigation.yaml"))
        );

        assert_eq!(errors[1].message, "Broken link detected in navigation");
        assert_eq!(
            errors[1].description,
            "Link /something/notexists/ points to an unknown file."
        );
        assert_eq!(errors[1].file, Some(PathBuf::from("/navigation.yaml")));
    }

    #[test]
    fn computes_size() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("---".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project\n")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("Some content!".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        assert_eq!(project.content_size_bytes, 38);
    }

    #[test]
    fn verifies_the_frontmatter_of_md_files_for_correct_yaml() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - not: valid
                  asdf
                ---

                # Hi
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let result = &project
            .verify(None, None)
            .expect_err("Project did not catch broken frontmatter");

        assert_eq!(result[0].message, "Invalid YAML syntax in frontmatter");
    }

    #[test]
    fn verifies_all_tabs_have_root_v2() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                tabs:
                  - label: Default
                    path: /
                  - label: Again
                    path: /foo
                    subtabs:
                      - label: Second
                        path: /foo/bar
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let errors = project.verify(None, None).unwrap_err();

        assert!(
            errors.iter().any(|e| e.message
                == "Missing root README.md for subtab \"Second\". Add a file at \"/foo/bar/README.md\"."),
            "No error for missing root readme"
        );
    }

    #[test]
    fn doesnt_verify_external_tabs_have_root() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Hi
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                tabs:
                  - label: Default
                    external: https://www.example.com
                  - label: Again
                    path: /
                    subtabs:
                      - label: Second
                        external: https://www.example.com
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        project.verify(None, None).unwrap();
    }

    #[test]
    fn it_gets_external_links_for_all_render_options() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                <Fragment if={@user_preferences.game == "Football"}>
                    [This is a football link](https://football.com)
                </Fragment>
                <Fragment if={@user_preferences.game == "Baseball"}>
                    [This is a baseball link](https://baseball.com)
                </Fragment>
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                user_preferences:
                  game:
                    label: Game
                    default: Football
                    values:
                      - Baseball
                      - Football
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let external_links = project.get_external_links();

        assert_eq!(
            external_links,
            vec![
                "https://baseball.com".to_string(),
                "https://football.com".to_string()
            ]
        );
    }

    #[test]
    fn busts_caches_when_asked() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("---".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project\n")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("![img](/_assets/cat.png)".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let page = project.get_page_by_uri_path("/").unwrap();
        let root = page
            .ast(Some(&RenderOptions {
                bust_image_caches: true,
                ..Default::default()
            }))
            .unwrap();
        let img = &root.as_markdown().unwrap().children[0].children[0];

        match &img.kind {
            crate::NodeKind::Image { url, .. } => {
                assert!(url.contains("cat.png?c=1"), "Image cache not busted");
            }
            _ => panic!("Not an image: {:#?}", img),
        }
    }

    #[test]
    fn can_be_asked_if_any_navigation_has_a_link_to_a_specific_page() {
        let files = vec![
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: User preference tab search
                user_preferences:
                  hobby:
                    label: Hobby
                    default: Football
                    values:
                      - Baseball
                      - Football

                tabs:
                  - label: Tab1
                    path: /
                  - label: Tab2
                    path: /tab2/
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Nav
                  items:
                    - href: /
                      label: Home
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("tab2").join(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Nav
                  items:
                    - href: ./bar.md
                      label: bar
                    - href: ./baz
                      label: baz
                    - href: ./foo.md
                      label: foo
                      show_if:
                        user_preferences:
                          hobby:
                            equals: Baseball
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("![img](/_assets/cat.png)".to_string()),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("other.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("tab2").join("foo.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("tab2").join("bar.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("tab2").join("baz.md"),
                content: InputContent::Text(String::new()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        assert!(project.navigation_has_link_to("/", None));
        assert!(project.navigation_has_link_to("/README.md", None));

        assert!(!project.navigation_has_link_to("/other", None));
        assert!(!project.navigation_has_link_to("/other.md", None));

        assert!(project.navigation_has_link_to("/tab2/bar.md", None));
        assert!(project.navigation_has_link_to("/tab2/bar", None));

        assert!(project.navigation_has_link_to("/tab2/baz.md", None));
        assert!(project.navigation_has_link_to("/tab2/baz", None));

        assert!(!project.navigation_has_link_to("/tab2/foo.md", None));
        assert!(!project.navigation_has_link_to("/tab2/foo", None));

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("hobby".to_owned(), "Baseball".to_owned());

        assert!(project.navigation_has_link_to("/tab2/foo.md", Some(&opts)));
        assert!(project.navigation_has_link_to("/tab2/foo", Some(&opts)));
    }

    #[test]
    fn v2_verifies_accent_color() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                ---
                title: An Project

                theme:
                  colors:
                    accent: lolnotacolor
                "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    Hi
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let opts = RenderOptions::default();

        let errors = project.verify(Some(&opts), None).unwrap_err();

        let error = &errors[0];

        assert!(
            &error.description.starts_with("Expected a HEX color code"),
            "Unexpected error: {:?}",
            error
        )
    }

    #[test]
    fn v2_supports_user_preferences() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                { @user_preferences.game }
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something

                user_preferences:
                  game:
                    label: Game
                    default: Football
                    values:
                      - Baseball
                      - Football
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let page = project.get_page_by_uri_path("/").unwrap();

        assert!(page.ast(None).is_ok());
        assert_eq!(project.verify(None, None), Ok(()));
    }

    #[test]
    fn v2_verifies_relative_links_inside_user_preference_conditionals() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                <Fragment if={@user_preferences.game == "Football"}>
                    [broken link](/what)
                </Fragment>
                <Fragment if={@user_preferences.game == "Baseball"}>
                    [broken link](/the)
                </Fragment>
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something

                user_preferences:
                  game:
                    label: Game
                    default: Football
                    values:
                      - Baseball
                      - Football
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let errors = project.verify(None, None).unwrap_err();
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].message, "Broken link detected");
        assert_eq!(errors[1].message, "Broken link detected");
    }

    #[test]
    fn v2_custom_components_with_slots_can_be_verified() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                    <Component.LandingPageCard heading="Foo">
                        Foo bar
                    </Component.LandingPageCard>
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("_components/landing-page-card.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    attributes:
                      - title: heading
                        required: true
                    ---

                    <div class="landing-page-card">
                      <div class="landing-page-card-header">
                      </div>
                      <Box>
                        <Slot />
                      </Box>
                    </div>
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let page = project.get_page_by_uri_path("/").unwrap();
        let result = page.ast(None);
        assert!(
            result.is_ok(),
            "Unable to render page. Got error: {:#?}",
            result
        );
        assert_eq!(project.verify(None, None), Ok(()));
    }

    #[test]
    fn v2_custom_components_with_syntax_errors_can_be_verified() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                    <Component.LandingPageCard heading="Foo">
                        Foo bar
                    </Component.LandingPageCard>
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("_components/landing-page-card.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    attributes:
                      - title: heading
                        required: true
                    ---

                    <div></BOB>
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let page = project.get_page_by_uri_path("/").unwrap();
        let result = page.ast(None);
        assert!(
            result.is_err(),
            "Component error not found. Got page: {:#?}",
            result
        );
        let _errors = project.verify(None, None).unwrap_err();
    }

    #[test]
    fn v2_custom_components_are_not_pages() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                    <Component.LandingPageCard heading="Foo">
                        Foo bar
                    </Component.LandingPageCard>
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("_components/landing-page-card.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    attributes:
                      - title: heading
                        required: true
                    ---

                    <div></BOB>
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        assert!(
            project
                .get_page_by_uri_path("/_components/landing-page-card")
                .is_none(),
            "Custom component parsed as regular page"
        );
    }

    #[test]
    fn v2_slot_can_only_be_used_in_components() {
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                    <Slot />
                "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let page = project.get_page_by_uri_path("/").unwrap();
        let error = page.ast(None).unwrap_err();

        assert_eq!(error.message, "Invalid slot");
        assert_eq!(
            error.description,
            indoc! { r#"
            `<Slot />` can only be used in components and topics

                1 │ <Slot />
                    ▲▲▲▲▲▲▲▲
                    └─ Invalid slot

        "#}
        );
    }

    #[test]
    fn v2_verify_returns_an_error_if_the_openapi_spec_contains_unknown_components_in_tag_description(
    ) {
        // BUG: DOC-1170
        let files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from("openapi.yaml"),
                content: InputContent::Text(
                    indoc! {r#"
                    openapi: "3.0.0"
                    info:
                      version: 1.0.0
                      title: Swagger Petstore
                      license:
                        name: MIT
                    servers:
                      - url: http://petstore.swagger.io/v1
                    tags:
                      - name: pets
                        description: THIS IS BROKEN <Nope>Hi</Nope>
                    paths:
                      /pets:
                        get:
                          summary: List all pets
                          operationId: listPets
                          tags:
                            - pets
                          parameters:
                            - name: limit
                              in: query
                              description: How many items to return at one time (max 100)
                              required: false
                              schema:
                                type: integer
                                maximum: 100
                                format: int32
                          responses:
                            '200':
                              description: A paged array of pets
                              headers:
                                x-next:
                                  description: A link to the next page of responses
                                  schema:
                                    type: string
                              content:
                                application/json:
                                  schema:
                                    $ref: '#/components/schemas/Pets'
                            default:
                              description: unexpected error
                              content:
                                application/json:
                                  schema:
                                    $ref: '#/components/schemas/Error'
                    components:
                      schemas:
                        Pet:
                          type: object
                          required:
                            - id
                            - name
                          properties:
                            id:
                              type: integer
                              format: int64
                            name:
                              type: string
                            tag:
                              type: string
                        Pets:
                          type: array
                          maxItems: 100
                          items:
                            $ref: '#/components/schemas/Pet'
                        Error:
                          type: object
                          required:
                            - code
                            - message
                          properties:
                            code:
                              type: integer
                              format: int32
                            message:
                              type: string
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Something
                "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Something
                open_api:
                  - spec_file: openapi.yaml
                    uri_prefix: /api
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = &Project::from_file_list(files).unwrap();

        assert!(
            project
                .get_page_by_uri_path("/api/pets")
                .unwrap()
                .ast(None)
                .is_err(),
            "Did not get error on broken OpenAPI spec"
        );

        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(
            errors[0].description,
            indoc! { r#"
            Unknown element "Nope"

                1 │ THIS IS BROKEN <Nope>Hi</Nope>
                                    ▲▲▲▲

        "#}
        );
    }

    #[test]
    fn checks_features_dont_exist() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    - heading: Something
                    "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                    ---
                    title: An Project
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("Hello".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let expected: Vec<String> = vec![];
        assert_eq!(project.check_features(), expected);
    }

    #[test]
    fn checks_features_custom_css_exists() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    - heading: Something
                    "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                    ---
                    title: An Project
                    styles:
                      - _assets/custom.css

                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("Hello".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        assert!(project.check_features().contains(&"custom_css".to_string()))
    }

    #[test]
    fn checks_features_user_preferences() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    - heading: Something
                    "#}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                    ---
                    title: An Project
                    user_preferences:
                      plan:
                        label: My Plan
                        default: Starter
                        values:
                          - Starter
                          - Growth
                          - Enterprise

                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("Hello".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        assert!(project
            .check_features()
            .contains(&"user_preferences".to_string()))
    }

    #[test]
    fn download_link_works_with_openapi_non_assets_folder() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <Link download href="openapi.json">Download</Link>
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r#"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        }
                      }
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        assert!(project
            .verify(Some(&RenderOptions::default()), None)
            .is_ok());
    }

    #[test]
    fn parses_openapi_schema_node() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <OpenAPISchema title="User" openapi_path="openapi.json" />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r##"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        },
                        "components": {
                          "schemas": {
                              "User": {
                                  "title": "User schema",
                                  "description": "# Hello, this is a title",
                                  "type": "object",
                                  "properties": {
                                      "id": {
                                          "type": "integer",
                                          "format": "int64"
                                      },
                                      "username": {
                                          "type": "string"
                                      }
                                  }
                              }
                          }
                        }
                      }
                "##}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        if let Ast::Markdown(root) = ast {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <OpenAPISchema title={User schema} expanded={true}>
                    <Heading1>
                        <Text>
                            Hello, this is a title
                        </Text>
                    </Heading1>
                </OpenAPISchema>
                "#}
                .to_string()
            );
        } else {
            panic!("Expected markdown AST");
        }
    }

    #[test]
    fn openapi_schemas_can_be_toggled_to_not_expanded() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <OpenAPISchema title="User" openapi_path="openapi.json" expanded={false} />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r##"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        },
                        "components": {
                          "schemas": {
                              "User": {
                                  "title": "User schema",
                                  "description": "# Hello, this is a title",
                                  "type": "object",
                                  "properties": {
                                      "id": {
                                          "type": "integer",
                                          "format": "int64"
                                      },
                                      "username": {
                                          "type": "string"
                                      }
                                  }
                              }
                          }
                        }
                      }
                "##}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        if let Ast::Markdown(root) = ast {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <OpenAPISchema title={User schema} expanded={false}>
                    <Heading1>
                        <Text>
                            Hello, this is a title
                        </Text>
                    </Heading1>
                </OpenAPISchema>
                "#}
                .to_string()
            );
        } else {
            panic!("Expected markdown AST");
        }
    }

    #[test]
    fn openapi_schema_node_error_when_schema_not_found() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <OpenAPISchema title="Foo" openapi_path="openapi.json" />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r##"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        },
                        "components": {
                          "schemas": {
                              "User": {
                                  "title": "User schema",
                                  "description": "# Hello, this is a title",
                                  "type": "object",
                                  "properties": {
                                      "id": {
                                          "type": "integer",
                                          "format": "int64"
                                      },
                                      "username": {
                                          "type": "string"
                                      }
                                  }
                              }
                          }
                        }
                      }
                "##}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();
        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let error = page.ast(Some(&opts)).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! { r#"
            OpenAPI schema with title "Foo" not found

                1 │ <OpenAPISchema title="Foo" openapi_path="openapi.json" />
                                          ▲▲▲

            "# }
            .to_string()
        );
    }

    #[test]
    fn openapi_schema_node_error_when_openapi_not_found() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <OpenAPISchema title="User" openapi_path="openapi.json" />
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let error = page.ast(Some(&opts)).unwrap_err();

        assert_str_eq!(
            error.description,
            indoc! { r#"
            Can't find an OpenAPI file with path "openapi.json"

                1 │ <OpenAPISchema title="User" openapi_path="openapi.json" />
                                                              ▲▲▲▲▲▲▲▲▲▲▲▲

            "# }
            .to_string()
        );
    }

    #[test]
    #[ignore]
    fn parses_openapi_schema_node_with_expanded() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <OpenAPISchema title="User" openapi_path="openapi.json" expanded={true} />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r##"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        },
                        "components": {
                          "schemas": {
                              "User": {
                                  "title": "User schema",
                                  "description": "# Hello, this is a title",
                                  "type": "object",
                                  "properties": {
                                      "id": {
                                          "type": "integer",
                                          "format": "int64"
                                      },
                                      "username": {
                                          "type": "string"
                                      }
                                  }
                              }
                          }
                        }
                      }
                "##}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        if let Ast::Markdown(root) = ast {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <OpenAPISchema title={User schema} openapi_path="openapi.json" expanded={true}>
                    <Heading1>
                        <Text>
                            Hello, this is a title
                        </Text>
                    </Heading1>
                </OpenAPISchema>
                "#}
                .to_string()
            );
        } else {
            panic!("Expected markdown AST");
        }
    }

    #[test]
    #[ignore]
    fn parses_openapi_schema_node_with_expanded_stringified() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: An Project
                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                    "# }
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                      <OpenAPISchema title="User" openapi_path="openapi.json" expanded="true" />
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(
                    indoc! {r##"
                    {
                        "openapi": "3.0.0",
                        "info": {
                          "version": "1.0.0",
                          "title": "Example code sample extensions case"
                        },
                        "paths": {
                          "/foo": {
                            "get": {
                              "description": "Foo GET",
                              "tags": [
                                "test"
                              ],
                              "parameters": [],
                              "responses": {},
                            }
                          }
                        },
                        "components": {
                          "schemas": {
                              "User": {
                                  "title": "User schema",
                                  "description": "# Hello, this is a title",
                                  "type": "object",
                                  "properties": {
                                      "id": {
                                          "type": "integer",
                                          "format": "int64"
                                      },
                                      "username": {
                                          "type": "string"
                                      }
                                  }
                              }
                          }
                        }
                      }
                "##}
                    .to_string(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        if let Ast::Markdown(root) = ast {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <OpenAPISchema title={User schema} expanded={true}>
                    <Heading1>
                        <Text>
                            Hello, this is a title
                        </Text>
                    </Heading1>
                </OpenAPISchema>
                "#}
                .to_string()
            );
        } else {
            panic!("Expected markdown AST");
        }
    }

    #[test]
    fn can_get_the_ast_for_a_virtual_page_in_a_project() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                ---
                title: An Project
                "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    # Hi
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();
        let ast = project
            .get_ast_mdx_fault_tolerant("hi", Path::new("example.md"), &opts)
            .unwrap();

        assert_str_eq!(
            ast.debug_string().unwrap(),
            indoc! { r#"
            <Paragraph>
                <Text>
                    hi
                </Text>
            </Paragraph>
            "# }
            .to_string()
        );
    }

    #[test]
    fn can_get_errors_for_a_virtual_page_in_a_project() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                ---
                title: An Project
                "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    # Hi
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let opts = RenderOptions::default();
        let result = project
            .get_ast_mdx_fault_tolerant("hi\n</>", Path::new("example.md"), &opts)
            .unwrap_err();

        let ast = result.0.unwrap();
        assert_eq!(ast.children.len(), 1);

        let errors = result.1;
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn can_offset_virtual_file_error_offsets_by_frontmatter() {
        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                ---
                title: An Project
                "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    # Hi
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let markdown = indoc! {r#"
        ---
        title: An Project
        ---

        # Hi

        </>
        "#
        };

        let opts = RenderOptions::default();
        let result = project
            .get_ast_mdx_fault_tolerant(markdown, Path::new("example.md"), &opts)
            .unwrap_err();

        let errors = result.1;
        assert_eq!(errors.len(), 1);

        let error = &errors[0];
        assert_eq!(error.position.as_ref().unwrap().start.row, 7);
        assert_eq!(error.position.as_ref().unwrap().start.col, 1);
        assert_eq!(error.position.as_ref().unwrap().start.byte_offset, 33);
        assert_eq!(error.position.as_ref().unwrap().end.row, 7);
        assert_eq!(error.position.as_ref().unwrap().end.col, 4);
        assert_eq!(error.position.as_ref().unwrap().end.byte_offset, 36);

        assert_eq!(
            "</>",
            &markdown[error.position.as_ref().unwrap().start.byte_offset
                ..error.position.as_ref().unwrap().end.byte_offset]
        );
    }

    #[test]
    fn verifies_vale_config_file_exists() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("---".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
              ---
              title: An Project

              vale:
                version: 2.30.0
                config_file_path: .vale.ini
              "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                  # Hi
                  "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(errors[0].message, "Couldn't find Vale configuration file.");
        assert_eq!(
            errors[0].description,
            "Expected a Vale configuration file at \".vale.ini\""
        );
    }

    #[test]
    fn verifies_vale_config_path() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("---".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
              ---
              title: An Project

              vale:
                version: 2.30.0
                config_file_path: .vale.inia
              "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                  # Hi
                  "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(".vale.inia"),
                content: InputContent::Text(
                    indoc! {r#"
                  # Hi
                  "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors.len(), 1);
        assert_eq!(
            errors[0].message,
            "A hidden vale configuration file must be named \".vale.ini\"."
        );
        assert_eq!(
            errors[0].description,
            "Use \".vale.ini\", or remove the \".\" from the start of the config file name \".vale.inia\"."
        );
    }

    #[test]
    fn creates_an_elasticlunr_search_index() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("---".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project\n")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# A heading\n\nA paragraph".to_string()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let index = project.search_index().unwrap();

        assert!(index.to_json().contains("A heading"));
        assert!(index.to_json().contains("A paragraph"));
    }
}
