use std::time::{SystemTime, UNIX_EPOCH};

/// Module that defines the interface for responses returned by the internal content API consumed
/// by venue.
///
/// Both Jaleo and the Desktop use these types in order to ensure we have a consistent API in both
/// platforms.
use crate::{
    breadcrumb::Breadcrumb,
    description_extractor::DescriptionExtractor,
    frontmatter::PageWidth,
    markdown_page::OnThisPageHeading,
    navigation::{Navigation, Section},
    settings::Settings,
    Ast, Error, PageHandle, Project as LibdoctaveProject, RenderOptions, Tab,
};

#[cfg(feature = "rustler")]
use rustler::{NifStruct, NifUnitEnum};

#[cfg(test)]
use ts_rs::TS;

#[derive(Debug, Clone)]
/// All the information required to send over the final response.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct ResponseContext {
    /// Render options for rendering Markdown
    pub options: RenderOptions,
    /// Information about the Jaleo-backed site. This is stubbed out in Desktop
    pub site: Site,
    /// Rewritten custom CSS files. Unfortunately required because we have to rewrite font
    /// links at runtime to point to the right S3 bucket.
    pub custom_css: Vec<String>,
    /// The URL we can load a favicon from
    pub favicon_url: Option<String>,
    /// The currently active version. None if we're in preview mode.
    pub active_version: Option<Version>,
    /// Build info. Stubbed in desktop.
    pub build: Build,
    /// Are we in live or preview mode.
    pub view_mode: ViewMode,
    /// Should we sign assets.
    pub sign_assets: bool,
    /// Debug information
    pub debug_info: DebugInfo,
}

impl Default for ResponseContext {
    fn default() -> Self {
        ResponseContext {
            options: RenderOptions::default(),
            site: Site::stub(),
            custom_css: vec![],
            favicon_url: None,
            active_version: None,
            build: Build::stub(),
            view_mode: ViewMode::Live,
            sign_assets: false,
            debug_info: DebugInfo::default(),
        }
    }
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifUnitEnum))]
#[serde(rename_all = "snake_case")]
pub enum ViewMode {
    Live,
    Preview,
    Desktop,
}

#[derive(Serialize, Debug, Clone)]
/// Information about the project beyond just the content itself. Required to render surrounding
/// information etc. This will change based on what page the reader is currently on.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct Project {
    /// Information about the Jaleo Site. This is injected from the outside.
    site: Site,
    /// What versions are currently visible to the user. None if we're in preview mode.
    active_version: Option<Version>,
    /// The doctave.yaml settings for the project.
    settings: Settings,
    /// What tabs do we have for the project
    tabs: Vec<Tab>,
    /// Index of the currently active version
    active_tab_index: Vec<usize>,
    /// The currently active navigation structure
    active_navigation: CurrentNavigation,
    /// Vec of CSS strings that need to be rendered
    custom_css: Vec<String>,
    /// The URL we can load a favicon from
    favicon_url: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
/// Information that comes from Jaleo. In the desktop, this will be stubbed out with default
/// values.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.ContentApi.Site")]
pub struct Site {
    /// The Jaleo site ID for this project
    pub id: usize,
    /// The Jaleo team ID for this project
    pub team_id: usize,
    /// The slug that represents the subdomain the project is hosted under
    pub slug: String,
    /// What versions have been configured for the site
    pub versions: Vec<Version>,
    /// Enabled integrations
    pub integrations: Integrations,
    /// If we are collecting feedback from reader)s
    pub collecting_feedback: bool,
    /// Canonical domain we can render as an HTML meta tag
    pub canonical_domain: String,
}

impl Site {
    pub fn stub() -> Self {
        Site {
            id: 0,
            team_id: 0,
            slug: "placeholder-project".to_string(),
            versions: vec![],
            integrations: Integrations::default(),
            collecting_feedback: false,
            canonical_domain: "https://www.example.com".to_string(),
        }
    }
}

impl Default for Site {
    fn default() -> Self {
        Site::stub()
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
/// Information about the current build. On desktop this will be stubbed out data.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.ContentApi.Build")]
pub struct Build {
    id: usize,
    inserted_at: String,
    git_branch: String,
    git_sha: String,
    manual_version_id: Option<usize>,
}

impl Build {
    pub fn stub() -> Self {
        Build {
            id: 0,
            inserted_at: "1970-01-01T00:00:00.000Z".to_string(),
            git_branch: "main".to_string(),
            git_sha: "64e09c0f863d6c119bd6af59ddaba998eaf95701d7c01aa3216ebab5ea9d003c".to_string(),
            manual_version_id: None,
        }
    }
}

#[derive(Serialize, Debug, Default, Clone)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.ContentApi.DebugInfo"
)]
pub struct DebugInfo {
    performance: Option<Vec<DebugInfoStep>>,
}

impl DebugInfo {
    pub fn start_performance(&mut self, label: &str) {
        if let Some(perf) = &mut self.performance {
            let start = SystemTime::now();
            let since_the_epoch = start
                .duration_since(UNIX_EPOCH)
                .expect("Time went backwards");
            perf.push(DebugInfoStep {
                label: label.to_string(),
                duration_ms: since_the_epoch.as_millis() as usize,
            });
        }
    }

    pub fn end_performance(&mut self, label: &str) {
        if let Some(perf) = &mut self.performance {
            if let Some(found) = perf.iter_mut().find(|entry| entry.label == label) {
                let end = SystemTime::now();
                let since_the_epoch = end.duration_since(UNIX_EPOCH).expect("Time went backwards");

                found.duration_ms = (since_the_epoch.as_millis() as usize) - found.duration_ms;
            }
        }
    }
}

#[derive(Serialize, Debug, Default, Clone)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.ContentApi.DebugInfoStep"
)]
pub struct DebugInfoStep {
    label: String,
    duration_ms: usize,
}

/// A response from the Doctave Content API with. This is returned from both Jaleo and the desktop
/// from the built-in content server.
#[derive(Serialize, Debug, Clone)]
#[serde(tag = "kind")]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[allow(clippy::large_enum_variant)]
pub enum ContentApiResponse {
    #[serde(rename = "content")]
    Content {
        page: CurrentPage,
        project: Project,
        build: Build,
        view_mode: ViewMode,
        sign_assets: bool,
        debug_info: DebugInfo,
    },
    #[serde(rename = "site_asleep")]
    SiteAsleep {
        http_status: u16,
        message: String,
        is_owner: bool,
        view_mode: ViewMode,
    },
    #[serde(rename = "password_auth_required")]
    PasswordAuthRequired {
        http_status: u16,
        auth_url: String,
        view_mode: ViewMode,
        settings: Settings,
        version_name: String,
        message: String,
        sign_assets: bool,
    },
    #[serde(rename = "private_site")]
    PrivateSite {
        http_status: u16,
        message: String,
        view_mode: ViewMode,
    },
    #[serde(rename = "build_not_found")]
    BuildNotFound {
        http_status: u16,
        message: String,
        view_mode: ViewMode,
    },
    #[serde(rename = "build_pending")]
    BuildPending {
        http_status: u16,
        message: String,
        view_mode: ViewMode,
    },
    #[serde(rename = "invalid_project")]
    InvalidProject {
        http_status: u16,
        message: String,
        errors: Vec<Error>,
        view_mode: ViewMode,
    },
    #[serde(rename = "unknown_error")]
    UnknownError {
        http_status: u16,
        message: String,
        view_mode: ViewMode,
    },
}

impl ContentApiResponse {
    pub fn response_status(&self) -> u16 {
        match self {
            ContentApiResponse::Content { page, .. } => match page {
                CurrentPage::NotFound { http_status, .. } => *http_status,
                CurrentPage::Error { http_status, .. } => *http_status,
                CurrentPage::Page { http_status, .. } => *http_status,
            },
            ContentApiResponse::SiteAsleep { http_status, .. } => *http_status,
            ContentApiResponse::PasswordAuthRequired { http_status, .. } => *http_status,
            ContentApiResponse::PrivateSite { http_status, .. } => *http_status,
            ContentApiResponse::BuildNotFound { http_status, .. } => *http_status,
            ContentApiResponse::BuildPending { http_status, .. } => *http_status,
            ContentApiResponse::InvalidProject { http_status, .. } => *http_status,
            ContentApiResponse::UnknownError { http_status, .. } => *http_status,
        }
    }

    pub fn site_asleep<S: Into<String>>(message: S, is_owner: bool, view_mode: ViewMode) -> Self {
        ContentApiResponse::SiteAsleep {
            message: message.into(),
            is_owner,
            http_status: 200,
            view_mode,
        }
    }
    pub fn password_auth_required<S: Into<String>>(
        project: &LibdoctaveProject,
        message: S,
        auth_url: S,
        version_name: S,
        view_mode: ViewMode,
        sign_assets: bool,
        opts: RenderOptions,
    ) -> Self {
        let mut settings = (*project.settings).clone();
        settings.rewrite_links(&opts, &project.assets);

        ContentApiResponse::PasswordAuthRequired {
            message: message.into(),
            settings,
            auth_url: auth_url.into(),
            version_name: version_name.into(),
            http_status: 401,
            view_mode,
            sign_assets,
        }
    }
    pub fn private_site<S: Into<String>>(message: S, view_mode: ViewMode) -> Self {
        ContentApiResponse::PrivateSite {
            message: message.into(),
            http_status: 404,
            view_mode,
        }
    }
    pub fn build_not_found<S: Into<String>>(message: S, view_mode: ViewMode) -> Self {
        ContentApiResponse::BuildNotFound {
            message: message.into(),
            http_status: 404,
            view_mode,
        }
    }
    pub fn build_pending<S: Into<String>>(message: S, view_mode: ViewMode) -> Self {
        ContentApiResponse::BuildPending {
            message: message.into(),
            http_status: 200,
            view_mode,
        }
    }
    pub fn invalid_project<S: Into<String>>(
        message: S,
        errors: Vec<Error>,
        view_mode: ViewMode,
    ) -> Self {
        ContentApiResponse::InvalidProject {
            errors,
            message: message.into(),
            http_status: 400,
            view_mode,
        }
    }
    pub fn unknown_error<S: Into<String>>(message: S, view_mode: ViewMode) -> Self {
        ContentApiResponse::UnknownError {
            message: message.into(),
            http_status: 500,
            view_mode,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(feature = "rustler", module = "Doctave.Libdoctave.ContentApi.Version")]
pub struct Version {
    id: usize,
    label: String,
    href: String,
    visibility: VersionVisibility,
    default: bool,
}

// Placeholder default implementation for testing
impl Default for Version {
    fn default() -> Self {
        Version {
            id: 123,
            label: "Default".to_string(),
            href: "/".to_string(),
            visibility: VersionVisibility::Public,
            default: true,
        }
    }
}

#[derive(Serialize, Debug, Clone, PartialEq)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifUnitEnum))]
pub enum VersionVisibility {
    Private,
    Public,
    PasswordProtected,
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "status")]
/// Current page, which may be a page that was rendered, of an error.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub enum CurrentPage {
    #[serde(rename = "ok")]
    Page {
        path: String,
        http_status: u16,
        ast: Ast,
        title: Option<String>,
        description: String,
        page_kind: String,
        breadcrumbs: Vec<Breadcrumb>,
        on_this_page_headings: Vec<OnThisPageHeading>,
        page_options: PageOptions,
    },
    #[serde(rename = "error")]
    Error {
        path: String,
        http_status: u16,
        errors: Vec<Error>,
        page_options: PageOptions,
        page_kind: String,
        title: String,
        description: String,
    },
    #[serde(rename = "not_found")]
    NotFound {
        path: String,
        http_status: u16,
        page_options: PageOptions,
    },
}

#[derive(Serialize, Debug, Clone)]
#[serde(tag = "status")]
/// Current page, which may be a page that was rendered, of an error.
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub enum CurrentNavigation {
    #[serde(rename = "ok")]
    Navigation { sections: Vec<Section> },
    #[serde(rename = "error")]
    Error { errors: Vec<Error> },
}

impl From<crate::Result<Navigation>> for CurrentNavigation {
    fn from(value: crate::Result<Navigation>) -> Self {
        match value {
            Ok(nav) => CurrentNavigation::Navigation {
                sections: nav.sections,
            },
            Err(error) => CurrentNavigation::Error {
                errors: vec![error],
            },
        }
    }
}

/// Internal convenience struct
struct Surrounding {
    active_tab: Vec<usize>,
    tabs: Vec<Tab>,
    navigation: crate::Result<Navigation>,
}

impl ContentApiResponse {
    pub fn content(
        page_handle: PageHandle,
        project: &LibdoctaveProject,
        ctx: ResponseContext,
    ) -> ContentApiResponse {
        let Surrounding {
            active_tab,
            tabs,
            navigation,
        } = Self::surrounding(page_handle.uri_path(), project, &ctx);

        let page = match page_handle.ast(Some(&ctx.options)) {
            Ok(ast) => CurrentPage::Page {
                path: page_handle.uri_path().to_string(),
                http_status: 200,
                title: page_handle.title().ok().flatten(),
                description: page_handle
                    .description()
                    .ok()
                    .flatten()
                    .unwrap_or(DescriptionExtractor::extract(&ast)),
                page_kind: if page_handle.is_openapi() {
                    "openapi".to_string()
                } else {
                    "markdown".to_string()
                },
                ast,
                breadcrumbs: page_handle.breadcrumbs(Some(&ctx.options)),
                on_this_page_headings: page_handle.on_this_page_headings(Some(&ctx.options)),
                page_options: PageOptions {
                    hide_navigation: page_handle.hide_navigation(),
                    hide_side_table_of_contents: page_handle.hide_side_table_of_contents(),
                    breadcrumbs: page_handle.show_breadcrumbs(),
                    page_width: page_handle.page_width(),
                    hidden_from_search: page_handle.hidden_from_search().unwrap_or(false),
                },
            },
            Err(error) => CurrentPage::Error {
                path: page_handle.uri_path().to_string(),
                http_status: 400,
                errors: vec![error],
                title: page_handle
                    .title()
                    .ok()
                    .flatten()
                    .unwrap_or(page_handle.fs_path().display().to_string()),
                description: page_handle.description().ok().flatten().unwrap_or_default(),
                page_kind: if page_handle.is_openapi() {
                    "openapi".to_string()
                } else {
                    "markdown".to_string()
                },
                page_options: PageOptions {
                    hide_navigation: page_handle.hide_navigation(),
                    hide_side_table_of_contents: page_handle.hide_side_table_of_contents(),
                    breadcrumbs: page_handle.show_breadcrumbs(),
                    page_width: page_handle.page_width(),
                    hidden_from_search: page_handle.hidden_from_search().unwrap_or(false),
                },
            },
        };

        let mut settings = (*project.settings).clone();
        settings.rewrite_links(&ctx.options, &project.assets);
        settings.resolve_paths(&ctx.options, page_handle.page.fs_path());

        ContentApiResponse::Content {
            page,
            project: Project {
                site: ctx.site,
                settings,
                tabs,
                active_tab_index: active_tab,
                active_version: ctx.active_version,
                custom_css: ctx.custom_css,
                favicon_url: ctx.favicon_url,
                active_navigation: navigation.into(),
            },
            build: ctx.build,
            view_mode: ctx.view_mode,
            sign_assets: ctx.sign_assets,
            debug_info: ctx.debug_info,
        }
    }

    pub fn page_not_found(
        uri_path: &str,
        project: &LibdoctaveProject,
        ctx: ResponseContext,
    ) -> ContentApiResponse {
        let Surrounding {
            active_tab,
            tabs,
            navigation,
        } = Self::surrounding(uri_path, project, &ctx);

        let parent_href = if !tabs.is_empty() && !active_tab.is_empty() {
            match tabs[active_tab[0]] {
                Tab::TabV1(ref tab) => tab.subtabs[active_tab[1]].href.as_str(),
                Tab::TabV2(ref tab) => tab.href.as_str(),
            }
        } else {
            "/"
        };

        // Try to fall back to the current subtab's page options, if available
        let page_options = if let Some(handle) = project.get_page_by_uri_path(parent_href) {
            PageOptions {
                hide_navigation: handle.hide_navigation(),
                hide_side_table_of_contents: handle.hide_side_table_of_contents(),
                breadcrumbs: handle.show_breadcrumbs(),
                page_width: handle.page_width(),
                hidden_from_search: handle.hidden_from_search().unwrap_or(false),
            }
        } else {
            PageOptions {
                hide_navigation: false,
                hide_side_table_of_contents: false,
                breadcrumbs: true,
                page_width: PageWidth::Prose,
                hidden_from_search: false,
            }
        };

        let mut settings = (*project.settings).clone();
        settings.rewrite_links(&ctx.options, &project.assets);

        ContentApiResponse::Content {
            page: CurrentPage::NotFound {
                path: uri_path.to_string(),
                http_status: 404,
                page_options,
            },
            project: Project {
                site: ctx.site,
                tabs,
                settings,
                active_tab_index: active_tab,
                active_version: ctx.active_version,
                custom_css: ctx.custom_css,
                favicon_url: ctx.favicon_url,
                active_navigation: navigation.into(),
            },
            build: ctx.build,
            view_mode: ctx.view_mode,
            sign_assets: ctx.sign_assets,
            debug_info: ctx.debug_info,
        }
    }

    fn surrounding(
        uri_path: &str,
        project: &LibdoctaveProject,
        ctx: &ResponseContext,
    ) -> Surrounding {
        let mut tabs = project
            .structure()
            .map(|s| s.tabs().clone())
            .unwrap_or_default();
        let mut active_tab = vec![];
        let matching_subtab_path = project.get_subtab_path_by_uri_path(uri_path);
        let navigation = project.navigation(
            Some(&ctx.options),
            matching_subtab_path
                .clone()
                .unwrap_or("/".to_string())
                .as_str(),
        );

        if let Some(subtab_path) = matching_subtab_path {
            'outer: for (tab_index, tab) in tabs.iter().enumerate() {
                match tab {
                    Tab::TabV1(tab) => {
                        for (subtab_index, subtab) in tab.subtabs.iter().enumerate() {
                            if subtab.path == subtab_path {
                                active_tab.push(tab_index);
                                active_tab.push(subtab_index);
                                break 'outer;
                            }
                        }
                    }
                    Tab::TabV2(tab) => {
                        for (subtab_index, subtab) in tab.subtabs.iter().enumerate() {
                            if subtab.href == subtab_path {
                                active_tab.push(tab_index);
                                active_tab.push(subtab_index);
                                break 'outer;
                            }
                        }

                        if tab.href == subtab_path {
                            active_tab.push(tab_index);
                            break 'outer;
                        }
                    }
                }
            }
        }

        fn prefix_link(url: &str, prefix: Option<&String>) -> String {
            match prefix {
                Some(prefix) => {
                    let mut rewrite = String::from(prefix.strip_suffix('/').unwrap_or(prefix));
                    rewrite.push('/');
                    rewrite.push_str(url.strip_prefix('/').unwrap_or(url));
                    rewrite
                }
                None => url.to_string(),
            }
        }

        for tab in &mut tabs {
            match tab {
                Tab::TabV1(tab) => {
                    tab.href = prefix_link(&tab.href, ctx.options.prefix_link_urls.as_ref());

                    for subtab in &mut tab.subtabs {
                        subtab.href =
                            prefix_link(&subtab.href, ctx.options.prefix_link_urls.as_ref());
                    }
                }
                Tab::TabV2(tab) => {
                    let prefix = ctx.options.prefix_link_urls.as_deref().unwrap_or("");
                    tab.prefix(prefix);
                }
            }
        }

        Surrounding {
            active_tab,
            tabs,
            navigation,
        }
    }
}

#[derive(Serialize, Debug, Clone, Default, PartialEq)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
#[cfg_attr(feature = "rustler", derive(NifStruct))]
#[cfg_attr(
    feature = "rustler",
    module = "Doctave.Libdoctave.ContentApi.Integrations"
)]
pub struct Integrations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hubspot_tracking_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ms_clarity_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ga_tracking_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub plausible_tracking: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intercom_app_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statuspage_id: Option<String>,
}

#[derive(Serialize, Debug, Clone)]
#[cfg_attr(test, derive(TS))]
#[cfg_attr(test, ts(export))]
pub struct PageOptions {
    pub hide_navigation: bool,
    pub hide_side_table_of_contents: bool,
    pub page_width: PageWidth,
    pub breadcrumbs: bool,
    pub hidden_from_search: bool,
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_str_eq;

    use crate::{
        settings::{FooterLink, HeaderLink, InternalLink},
        InputContent, InputFile, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME, STRUCTURE_FILE_NAME,
    };

    use std::path::{Path, PathBuf};

    use super::*;

    #[test]
    fn basic() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(
                    "---\ntitle: An Project\ndoctave_version: 2",
                )),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/foo/bar", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page { .. },
                    ..
                }
            ),
            "Unexpected response {:#?}",
            response
        );

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "content");
        assert_eq!(as_json["page"]["status"], "ok");
        assert_eq!(as_json["page"]["breadcrumbs"][0]["text"], "Guides");
        assert_eq!(as_json["project"]["active_navigation"]["status"], "ok");
        assert_eq!(as_json["project"]["settings"]["version"], "2");
    }

    #[test]
    fn prefix_structure_yaml_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(STRUCTURE_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                tabs:
                  - label: "Guides"
                    subtabs:
                      - label: "Getting started"
                        path: "/"
                  - label: "Fizz"
                    subtabs:
                      - label: "Getting started"
                        path: "/fizz/"
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("foo").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                options: RenderOptions {
                    prefix_link_urls: Some("/dev".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content {
                project: Project { tabs, .. },
                ..
            } => {
                for tab in tabs {
                    match tab {
                        Tab::TabV1(tab) => {
                            assert!(tab.href.starts_with("/dev"), "Tab no prefixed: {:?}", tab);

                            for subtab in tab.subtabs {
                                assert!(
                                    subtab.href.starts_with("/dev"),
                                    "Subtab no prefixed: {:?}",
                                    subtab
                                );
                            }
                        }
                        Tab::TabV2(tab) => {
                            panic!("Unexpected tab: {:?}", tab)
                        }
                    }
                }
            }
            _ => panic!("Unexpected response: {:#?}", response),
        }
    }

    #[test]
    fn prefix_structure_yaml_links_tabs_v2() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                tabs:
                  - label: "Getting started"
                    path: "/foo/"
                  - label: "Fizz"
                    path: "/fizz/"
                    subtabs:
                      - label: "Getting started"
                        path: "/fizz/bar"
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("foo").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                options: RenderOptions {
                    prefix_link_urls: Some("/dev".to_string()),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content {
                project: Project { tabs, .. },
                ..
            } => {
                let tab1 = tabs
                    .iter()
                    .find(|tab| {
                        let v2 = tab.get_v2().unwrap();
                        v2.label == "Getting started"
                    })
                    .unwrap();
                let tab2 = tabs
                    .iter()
                    .find(|tab| {
                        let v2 = tab.get_v2().unwrap();
                        v2.label == "Fizz"
                    })
                    .unwrap();

                assert_eq!(tab1.get_v2().unwrap().href, "/dev/foo");
                assert_eq!(tab2.get_v2().unwrap().href, "/dev/fizz");

                let subtab1 = tab2
                    .get_v2()
                    .unwrap()
                    .subtabs
                    .iter()
                    .find(|subtab| subtab.label == "Getting started")
                    .unwrap();

                assert_eq!(subtab1.href, "/dev/fizz/bar");
            }
            _ => panic!("Unexpected response: {:#?}", response),
        }
    }

    #[test]
    fn active_tab_index_v2() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                tabs:
                  - label: "Getting started"
                    path: "/foo"
                  - label: "Fizz"
                    path: "/"
                    subtabs:
                      - label: "Getting started"
                        path: "/fizz"
                      - label: "Second"
                        path: "/"
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("foo").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path("/foo", ResponseContext::default());

        match response {
            ContentApiResponse::Content {
                project: Project {
                    active_tab_index, ..
                },
                ..
            } => {
                assert_eq!(active_tab_index, vec![0])
            }
            _ => panic!("Unexpected response: {:#?}", response),
        }
    }

    #[test]
    fn active_tab_index_subtab_as_root_v2() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz/README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                tabs:
                  - label: "Getting started"
                    path: "/foo"
                  - label: "Fizz"
                    path: "/"
                    subtabs:
                      - label: "Getting started"
                        path: "/fizz"
                      - label: "Second"
                        path: "/"
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("foo").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("fizz").join(Path::new(NAVIGATION_FILE_NAME)),
                content: InputContent::Text(String::new()),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path("/", ResponseContext::default());

        match response {
            ContentApiResponse::Content {
                project: Project {
                    active_tab_index, ..
                },
                ..
            } => {
                assert_eq!(active_tab_index, vec![1, 1])
            }
            _ => panic!("Unexpected response: {:#?}", response),
        }
    }

    #[test]
    fn page_not_found() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/dont-exist", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound {
                        page_options: PageOptions {
                            hide_navigation: false,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                },
            ),
            "Unexpected response {:#?}",
            response
        );

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "content");
        assert_eq!(as_json["page"]["status"], "not_found");
    }

    #[test]
    fn page_not_found_falls_back_to_current_tabs_page_options_v2() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("foobar".to_string()),
            },
            InputFile {
                path: PathBuf::from("foo/README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                hide_navigation: true
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                tabs:
                  - label: Guides
                    path: /
                  - label: Foo
                    path: /foo
                "# })),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/dont-exist", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound {
                        page_options: PageOptions {
                            hide_navigation: false,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden: {:#?}",
            response
        );

        let subtab_response =
            project.get_content_response_by_uri_path("/foo/dont-exist", ResponseContext::default());

        assert!(
            matches!(
                subtab_response,
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound {
                        page_options: PageOptions {
                            hide_navigation: true,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden: {:#?}",
            subtab_response
        );
    }

    #[test]
    fn page_not_found_falls_back_to_current_suntabs_page_options() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("foobar".to_string()),
            },
            InputFile {
                path: PathBuf::from("foo/README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                hide_navigation: true
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(STRUCTURE_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                tabs:
                  - label: "Guides"
                    subtabs:
                      - label: "Getting started"
                        path: "/"
                  - label: "Foo"
                    subtabs:
                      - label: "Getting started"
                        path: "/foo/"
                "#})),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/dont-exist", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound {
                        page_options: PageOptions {
                            hide_navigation: false,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation is hidden: {:#?}",
            response
        );

        let subtab_response =
            project.get_content_response_by_uri_path("/foo/dont-exist", ResponseContext::default());

        assert!(
            matches!(
                subtab_response,
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound {
                        page_options: PageOptions {
                            hide_navigation: true,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden: {:#?}",
            subtab_response
        );
    }

    #[test]
    fn hide_navigation_from_frontmatter() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                navigation: false
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                hide_navigation: true
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                "# })),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                        - heading: "Guides"
                        "#})),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path("/", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page {
                        page_options: PageOptions {
                            hide_navigation: true,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden with hide_navigation key: {:#?}",
            response
        );

        let response = project.get_content_response_by_uri_path("/foo", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page {
                        page_options: PageOptions {
                            hide_navigation: true,
                            hide_side_table_of_contents: false,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden with navigation key: {:#?}",
            response
        );
    }

    #[test]
    fn hide_toc_from_frontmatter() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                toc: false
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                hide_side_table_of_contents: true
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                "# })),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                        - heading: "Guides"
                        "#})),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path("/", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page {
                        page_options: PageOptions {
                            hide_navigation: false,
                            hide_side_table_of_contents: true,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden with hide_navigation key: {:#?}",
            response
        );

        let response = project.get_content_response_by_uri_path("/foo", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page {
                        page_options: PageOptions {
                            hide_navigation: false,
                            hide_side_table_of_contents: true,
                            breadcrumbs: true,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Navigation not hidden with navigation key: {:#?}",
            response
        );
    }

    #[test]
    fn page_width_from_frontmatter() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                page_width: full
                ---

                Hi
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! { r#"
                ---
                title: An Project
                version: 2
                "# })),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                        - heading: "Guides"
                        "#})),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project.get_content_response_by_uri_path("/", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page {
                        page_options: PageOptions {
                            page_width: PageWidth::Full,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Page width not full",
        );

        let response = project.get_content_response_by_uri_path("/foo", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page {
                        page_options: PageOptions {
                            page_width: PageWidth::Prose,
                            ..
                        },
                        ..
                    },
                    ..
                }
            ),
            "Page width not defaulting to prose",
        );
    }

    #[test]
    fn error_on_page() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("{% bad liquid ")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/foo/bar", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Error { .. },
                    ..
                }
            ),
            "Unexpected response {:#?}",
            response
        );

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "content");
        assert_eq!(as_json["page"]["status"], "error");
        assert_eq!(
            as_json["page"]["errors"][0]["message"],
            "Error parsing liquid template"
        );
    }

    #[test]
    fn bad_navigation() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("{% bad liquid ")),
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
                - fizz: foo/bar.md
            "#})),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/foo/bar", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    project: Project {
                        active_navigation: CurrentNavigation::Error { .. },
                        ..
                    },
                    ..
                }
            ),
            "Unexpected response {:#?}",
            response
        );

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "content");
        assert_eq!(as_json["project"]["active_navigation"]["status"], "error");
        assert_eq!(
            as_json["project"]["active_navigation"]["errors"][0]["message"],
            "Invalid navigation.yaml"
        );
    }

    #[test]
    fn busted_project() {
        // If we can't open a project, we'll just have an error. So lets construct a response from
        // that.
        let error = crate::Error {
            code: Error::MISSING_DOCTAVE_YAML,
            message: "Look! An error!".to_string(),
            description: "More info about the error".to_string(),
            file: None,
            position: None,
        };

        let response = ContentApiResponse::InvalidProject {
            http_status: 400,
            message: "Some error about the project".to_string(),
            errors: vec![error],
            view_mode: ViewMode::Live,
        };

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "invalid_project");
        assert_eq!(as_json["errors"][0]["code"], Error::MISSING_DOCTAVE_YAML);
    }

    #[test]
    fn autocompute_title_and_description() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("Hello, world")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from(indoc! {r#"
                ---
                meta:
                   description: An description
                ---
                "#})),
            },
            InputFile {
                path: PathBuf::from("foo.md"),
                content: InputContent::Text(String::from(indoc! {r#"
                ---
                title: Manual title
                ---
                "#})),
            },
            InputFile {
                path: PathBuf::from("buzz.md"),
                content: InputContent::Text(String::from(indoc! {r#"
                "#})),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        match project.get_content_response_by_uri_path("/", ResponseContext::default()) {
            ContentApiResponse::Content { page, .. } => match page {
                CurrentPage::Page {
                    title, description, ..
                } => {
                    assert_eq!(description, "Hello, world");
                    assert_eq!(title, None);
                }
                _ => panic!(),
            },
            _ => panic!(),
        }

        match project.get_content_response_by_uri_path("/foo/bar", ResponseContext::default()) {
            ContentApiResponse::Content { page, .. } => match page {
                CurrentPage::Page { description, .. } => {
                    assert_eq!(description, "An description");
                }
                _ => panic!(),
            },
            _ => panic!(),
        }

        match project.get_content_response_by_uri_path("/foo", ResponseContext::default()) {
            ContentApiResponse::Content { page, .. } => match page {
                CurrentPage::Page { title, .. } => {
                    assert_eq!(title, Some("Manual title".to_string()));
                }
                _ => panic!(),
            },
            _ => panic!(),
        }

        match project.get_content_response_by_uri_path("/buzz", ResponseContext::default()) {
            ContentApiResponse::Content { page, .. } => match page {
                CurrentPage::Page { title, .. } => {
                    assert_eq!(title, Some("Buzz".to_string()));
                }
                _ => panic!(),
            },
            _ => panic!(),
        }
    }

    #[test]
    fn injects_build_from_context() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let expected_build = Build {
            id: 123,
            git_sha: "FFFFFF".to_string(),
            git_branch: "main".to_string(),
            inserted_at: "2024-03-12T08:35:14Z".to_string(),
            manual_version_id: None,
        };

        let response = project.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                build: expected_build.clone(),
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref build, .. } => {
                assert_eq!(
                    build, &expected_build,
                    "Unexpected build in response {:#?}",
                    response
                );
            }
            _ => panic!("Unexpected build in response {:#?}", response),
        }
    }

    #[test]
    fn rewrites_settings_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                title: Foo
                logo:
                  src: _assets/logo.png
                "#})),
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

        let proj = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = proj.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                options: RenderOptions {
                    // NOTE: Jaleo always prefixes asset links with slash
                    link_rewrites: [("/_assets/logo.png".to_string(), "new-logo".to_string())]
                        .into(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert_eq!(
                    &project.settings.logo().unwrap().src,
                    Path::new("new-logo"),
                    "Logo src not rewritten",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }

    #[test]
    fn cache_busts_logos() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                title: Foo
                logo:
                  src: _assets/logo.png
                  src_dark: _assets/logo_dark.png
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let proj = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = proj.get_content_response_by_uri_path(
            "/",
            ResponseContext {
                options: RenderOptions {
                    bust_image_caches: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert!(
                    &project
                        .settings
                        .logo()
                        .unwrap()
                        .src
                        .display()
                        .to_string()
                        .contains("?c="),
                    "Logo src not cache busted",
                );
                assert!(
                    &project
                        .settings
                        .logo()
                        .unwrap()
                        .src_dark
                        .as_ref()
                        .unwrap()
                        .display()
                        .to_string()
                        .contains("?c="),
                    "Logo src_dark not cache busted",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }

    #[test]
    fn image_cache_buster_from_key() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("_assets/logo.png"),
                content: InputContent::Binary(String::from("light")),
            },
            InputFile {
                path: PathBuf::from("_assets/logo_dark.png"),
                content: InputContent::Binary(String::from("dark")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                title: Foo
                logo:
                  src: _assets/logo.png
                  src_dark: _assets/logo_dark.png
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let proj = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = proj.get_content_response_by_uri_path(
            "/",
            ResponseContext {
                options: RenderOptions {
                    bust_image_caches: true,
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert!(
                    &project
                        .settings
                        .logo()
                        .unwrap()
                        .src
                        .display()
                        .to_string()
                        .contains("?c=11834040495846994434"),
                    "Logo src not cache busted",
                );
                assert!(
                    &project
                        .settings
                        .logo()
                        .unwrap()
                        .src_dark
                        .as_ref()
                        .unwrap()
                        .display()
                        .to_string()
                        .contains("?c=6183001633438668099"),
                    "Logo src_dark not cache busted",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }

    #[test]
    fn content_image_cache_bust_with_key() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("![good img](/_assets/asdf.png)")),
            },
            InputFile {
                path: PathBuf::from("_assets/asdf.png"),
                content: InputContent::Binary(String::from("")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(
                    "---\ntitle: An Project\ndoctave_version: 2",
                )),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let mut opts = RenderOptions::default();
        let mut ctx = ResponseContext::default();

        opts.bust_image_caches = true;

        ctx.options = opts;

        if let ContentApiResponse::Content {
            page:
                CurrentPage::Page {
                    ast: Ast::Markdown(root),
                    ..
                },
            ..
        } = project.get_content_response_by_uri_path("/", ctx)
        {
            assert_str_eq!(
                root.debug_string().unwrap(),
                indoc! { r#"
                <Paragraph>
                    <Image url={/_assets/asdf.png?c=15130871412783076140} alt={good img} />
                </Paragraph>
                "# }
            )
        } else {
            panic!("Unexpected response {:#?}", project);
        }
    }

    #[test]
    fn injects_version_from_context() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let expected_versions = vec![
            Version {
                id: 123,
                label: "V1".to_string(),
                href: "/V1".to_string(),
                visibility: VersionVisibility::Public,
                default: true,
            },
            Version {
                id: 124,
                label: "V2".to_string(),
                href: "/V2".to_string(),
                visibility: VersionVisibility::Private,
                default: false,
            },
            Version {
                id: 125,
                label: "V3".to_string(),
                href: "/V3".to_string(),
                visibility: VersionVisibility::PasswordProtected,
                default: false,
            },
        ];

        let response = project.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                site: Site {
                    versions: expected_versions.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert_eq!(
                    project.site.versions, expected_versions,
                    "Unexpected versions in response {:#?}",
                    response
                );
            }
            _ => panic!("Unexpected build in response {:#?}", response),
        }
    }

    #[test]
    fn injects_integrations_from_context() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let expected_integrations = Integrations {
            ga_tracking_code: Some("google".to_string()),
            intercom_app_id: Some("intercom".to_string()),
            statuspage_id: Some("statuspage".to_string()),
            hubspot_tracking_code: Some("hubspot".to_string()),
            ms_clarity_code: Some("ms_clarity".to_string()),
            plausible_tracking: Some(true),
        };

        let response = project.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                site: Site {
                    integrations: expected_integrations.clone(),
                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert_eq!(
                    project.site.integrations, expected_integrations,
                    "Unexpected integrationss in response {:#?}",
                    response
                );
            }
            _ => panic!("Unexpected build in response {:#?}", response),
        }
    }

    #[test]
    fn injects_css_from_context() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let expected_css = vec!["some css".to_string()];

        let response = project.get_content_response_by_uri_path(
            "/foo/bar",
            ResponseContext {
                custom_css: expected_css.clone(),
                ..Default::default()
            },
        );

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert_eq!(
                    project.custom_css, expected_css,
                    "Unexpected css in response {:#?}",
                    response
                );
            }
            _ => panic!("Unexpected build in response {:#?}", response),
        }
    }

    #[test]
    fn http_status_codes() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](/)")),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        assert!(
            matches!(
                project
                    .get_content_response_by_uri_path("/not/a/page/", ResponseContext::default()),
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound {
                        http_status: 404,
                        ..
                    },
                    ..
                }
            ),
            "Unexpected http status"
        );

        assert!(
            matches!(
                ContentApiResponse::site_asleep("Site asleep", false, ViewMode::Live),
                ContentApiResponse::SiteAsleep {
                    http_status: 200,
                    ..
                }
            ),
            "Unexpected http status"
        );

        assert!(
            matches!(
                ContentApiResponse::password_auth_required(
                    &LibdoctaveProject::default(),
                    "Site asleep",
                    "https://auth.me",
                    "Default",
                    ViewMode::Live,
                    false,
                    RenderOptions::default()
                ),
                ContentApiResponse::PasswordAuthRequired {
                    http_status: 401,
                    ..
                }
            ),
            "Unexpected http status"
        );

        assert!(
            matches!(
                ContentApiResponse::private_site("Not found", ViewMode::Live),
                ContentApiResponse::PrivateSite {
                    http_status: 404,
                    ..
                }
            ),
            "Unexpected http status"
        );

        assert!(
            matches!(
                ContentApiResponse::build_not_found("Not found", ViewMode::Live),
                ContentApiResponse::BuildNotFound {
                    http_status: 404,
                    ..
                }
            ),
            "Unexpected http status"
        );

        assert!(
            matches!(
                ContentApiResponse::invalid_project("Error", vec![], ViewMode::Live),
                ContentApiResponse::InvalidProject {
                    http_status: 400,
                    ..
                }
            ),
            "Unexpected http status"
        );
    }

    #[test]
    fn breadcrumbs_enabled_by_default() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                ---

                Hello
                "#}
                    .to_string(),
                ),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/foo/bar", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page { .. },
                    ..
                }
            ),
            "Unexpected response {:#?}",
            response
        );

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "content");
        assert_eq!(as_json["page"]["status"], "ok");
        assert_eq!(as_json["page"]["page_options"]["breadcrumbs"], true);
        assert_eq!(as_json["project"]["active_navigation"]["status"], "ok");
    }

    #[test]
    fn disabled_breadcrumbs() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                breadcrumbs: false
                ---

                Hello
                "#}
                    .to_string(),
                ),
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

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response =
            project.get_content_response_by_uri_path("/foo/bar", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::Page { .. },
                    ..
                }
            ),
            "Unexpected response {:#?}",
            response
        );

        let as_json = serde_json::to_value(&response).unwrap();

        assert_eq!(as_json["kind"], "content");
        assert_eq!(as_json["page"]["status"], "ok");
        assert_eq!(as_json["page"]["page_options"]["breadcrumbs"], false);
    }

    #[test]
    fn doc_1174_we_are_not_guaranteed_to_have_an_active_tab_on_404() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("developer-reference").join("README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("developer-reference").join("onboarding-guide.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("developer-reference").join(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    - heading: Developer reference
                      items:
                        - label: Onboarding guide
                          href: /onboarding-guide.md
                    "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                    ---
                    title: mmob
                    version: 2

                    tabs:
                      - label: Developer reference
                        path: /developer-reference
                "#}
                    .to_owned(),
                ),
            },
        ];

        let project = LibdoctaveProject::from_file_list(file_list).unwrap();

        let response = project
            .get_content_response_by_uri_path("/onboarding-guide", ResponseContext::default());

        assert!(
            matches!(
                response,
                ContentApiResponse::Content {
                    page: CurrentPage::NotFound { .. },
                    ..
                }
            ),
            "Unexpected response {:#?}",
            response
        );
    }

    #[test]
    fn webbifies_internal_header_link() {
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
                path: PathBuf::from("docs/foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Foo!
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
                      href: "/docs/foo.md"
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = LibdoctaveProject::from_file_list(files.clone()).unwrap();

        let ctx = ResponseContext {
            options: RenderOptions {
                webbify_internal_urls: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let response = project.get_content_response_by_uri_path("/", ctx);

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert!(
                    &project.settings.header().unwrap().links.iter().any(|l| {
                        match l {
                            HeaderLink::Internal(InternalLink { href, .. }) => href == "/docs/foo",
                            _ => false,
                        }
                    }),
                    "did not webbify header internal link",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }

    #[test]
    fn webbifies_internal_footer_link() {
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
                path: PathBuf::from("docs/foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Foo!
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
                doctave_version: 2
                footer:
                  links:
                    - label: "Doctave"
                      href: "/docs/foo.md"
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = LibdoctaveProject::from_file_list(files.clone()).unwrap();

        let ctx = ResponseContext {
            options: RenderOptions {
                webbify_internal_urls: true,
                ..Default::default()
            },
            ..Default::default()
        };
        let response = project.get_content_response_by_uri_path("/", ctx);

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert!(
                    &project.settings.footer().unwrap().links.iter().any(|l| {
                        match l {
                            FooterLink::Internal(InternalLink { href, .. }) => href == "/docs/foo",
                            _ => false,
                        }
                    }),
                    "did not webbify footer internal link",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }

    #[test]
    fn rewrites_header_download_links() {
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
                path: PathBuf::from("docs/foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Foo!
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
                doctave_version: 2
                header:
                  links:
                    - label: "Doctave"
                      href: "_assets/foo.png"
                      download: true
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = LibdoctaveProject::from_file_list(files.clone()).unwrap();

        let ctx = ResponseContext {
            options: RenderOptions {
                // NOTE: Jaleo always prefixes asset links with slash
                link_rewrites: [("/_assets/foo.png".to_string(), "new-foo".to_string())].into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let response = project.get_content_response_by_uri_path("/", ctx);

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert!(
                    &project.settings.header().unwrap().links.iter().any(|l| {
                        match l {
                            HeaderLink::Internal(InternalLink { href, .. }) => href == "new-foo",
                            _ => false,
                        }
                    }),
                    "did not rewrite header download link",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }

    #[test]
    fn rewrites_footer_download_links() {
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
                path: PathBuf::from("docs/foo.md"),
                content: InputContent::Text(
                    indoc! {r#"
                # Foo!
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
                doctave_version: 2
                footer:
                  links:
                    - label: "Doctave"
                      href: "_assets/foo.png"
                      download: true
                "#}
                    .to_string(),
                ),
            },
        ];

        let project = LibdoctaveProject::from_file_list(files.clone()).unwrap();

        let ctx = ResponseContext {
            options: RenderOptions {
                // NOTE: Jaleo always prefixes asset links with slash
                link_rewrites: [("/_assets/foo.png".to_string(), "new-foo".to_string())].into(),
                ..Default::default()
            },
            ..Default::default()
        };
        let response = project.get_content_response_by_uri_path("/", ctx);

        match response {
            ContentApiResponse::Content { ref project, .. } => {
                assert!(
                    &project.settings.footer().unwrap().links.iter().any(|l| {
                        match l {
                            FooterLink::Internal(InternalLink { href, .. }) => href == "new-foo",
                            _ => false,
                        }
                    }),
                    "did not rewrite footer download link",
                );
            }
            _ => panic!("Unexpected response {:#?}", response),
        }
    }
}
