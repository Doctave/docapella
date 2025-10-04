use std::collections::HashSet;
use std::path::PathBuf;
use std::{collections::HashMap, path::Path};

use crate::render_context::RenderContext;
use crate::{markdown, page_kind::PageKind, project::Project, Error, Result};
use serde::{Deserialize, Serialize};

/// Build the navigation structure.
///
/// Takes as input the Yaml file that describes the navigation, render options
/// used to e.g. filter by user preferences, and a reference to the project
/// itself, which can be used to generate OpenAPI links.
///
/// Returns an error if the navigation structure could not be built.
pub(crate) fn build(input: &str, ctx: &RenderContext, project: &Project) -> Result<Navigation> {
    let descriptions = parse_description(input)?;
    let mut sections = Vec::new();

    for desc in descriptions {
        if let Some(section) = desc.resolve(ctx, project) {
            sections.push(section);
        }
    }

    Ok(Navigation { sections })
}

/// Verify that the navigation is properly formed. Meaning it doesn't reference
/// unknown OpenAPI specs, user preferences, etc.
///
/// More specifically, these are contextual errors, not syntactic errors.
pub(crate) fn verify(input: &str, project: &Project) -> Vec<Error> {
    match parse_description(input) {
        Ok(sections) => {
            let mut errors = vec![];

            for section in &sections {
                for error in section.verify(project) {
                    errors.push(error);
                }
            }

            errors
        }
        Err(error) => vec![error],
    }
}

fn parse_description(input: &str) -> Result<Vec<SectionDescription>> {
    serde_yaml::from_str(input).map_err(|e| {
        Error::from_serde_yaml(
            e,
            Error::INVALID_NAVIGATION,
            "Invalid navigation.yaml".to_owned(),
            Some(PathBuf::from("navigation.yaml")),
        )
    })
}

#[derive(Serialize, PartialEq, Clone, Debug)]
pub struct Navigation {
    pub sections: Vec<Section>,
}

impl Navigation {
    pub fn new(sections: Vec<Section>) -> Self {
        Navigation { sections }
    }

    pub fn has_link_to(&self, uri_path: &str) -> bool {
        normalize_link(uri_path) == "/" || self.sections.iter().any(|s| s.has_link_to(uri_path))
    }

    pub(crate) fn gather_links(&self) -> Vec<String> {
        self.sections
            .iter()
            .flat_map(|s| s.gather_links())
            .collect::<Vec<_>>()
    }
}

impl std::ops::Deref for Navigation {
    type Target = Vec<Section>;

    fn deref(&self) -> &Self::Target {
        &self.sections
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct Section {
    pub heading: Option<String>,
    pub collapsed: bool,
    pub collapsible: bool,
    pub items: Vec<Item>,
}

impl Section {
    pub fn has_link_to(&self, uri_path: &str) -> bool {
        self.items.iter().any(|i| i.has_link_to(uri_path))
    }

    pub(crate) fn gather_links(&self) -> Vec<String> {
        self.items
            .iter()
            .flat_map(gather_links_from_item)
            .collect::<Vec<_>>()
    }
}

fn gather_links_from_item(item: &Item) -> Vec<String> {
    match item {
        Item::Subheading { items, .. } => items
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .flat_map(gather_links_from_item)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default(),

        Item::Link { href, items, .. } => {
            let mut out = items
                .as_ref()
                .map(|items| {
                    items
                        .iter()
                        .flat_map(gather_links_from_item)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            if let Some(href) = href {
                if markdown::parser::parse_internal_link(href).is_some() {
                    out.push(href.clone());
                }
            }

            out
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy, Serialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    #[serde(rename = "del")]
    Delete,
    #[serde(rename = "opt")]
    Option,
    Head,
    Trace,
    #[serde(rename = "event")]
    WebHook,
}

impl HttpMethod {
    fn from_str(s: &str) -> Option<HttpMethod> {
        match s.to_lowercase().as_str() {
            "get" => Some(HttpMethod::Get),
            "post" => Some(HttpMethod::Post),
            "put" => Some(HttpMethod::Put),
            "patch" => Some(HttpMethod::Patch),
            "delete" => Some(HttpMethod::Delete),
            "option" => Some(HttpMethod::Option),
            "head" => Some(HttpMethod::Head),
            "trace" => Some(HttpMethod::Trace),
            "webhook" => Some(HttpMethod::WebHook),
            _ => None,
        }
    }
}

impl std::fmt::Display for HttpMethod {
    fn fmt(&self, fmt: &mut std::fmt::Formatter) -> std::fmt::Result {
        use HttpMethod::*;
        let name = match self {
            Get => "get",
            Post => "post",
            Put => "put",
            Patch => "patch",
            Delete => "del",
            Option => "opt",
            Head => "head",
            Trace => "trace",
            WebHook => "webhook",
        };
        fmt.write_str(name)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind")]
pub enum Item {
    #[serde(rename = "subheading")]
    Subheading {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        collapsed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        collapsible: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<Vec<Item>>,
    },
    #[serde(rename = "link")]
    Link {
        label: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        external_href: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        href: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
        #[serde(skip_serializing_if = "Option::is_none")]
        collapsed: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        collapsible: Option<bool>,
        #[serde(skip_serializing_if = "Option::is_none")]
        http_method: Option<HttpMethod>,
        #[serde(skip_serializing_if = "Option::is_none")]
        items: Option<Vec<Item>>,
    },
}

fn normalize_link(link: &str) -> String {
    format!("/{}", link.strip_prefix('/').unwrap_or(link))
}

fn matches_link(uri_or_fs_path: &str, other_uri_or_fs_path: &str) -> bool {
    crate::uri_to_fs_path(&normalize_link(uri_or_fs_path))
        == crate::uri_to_fs_path(&normalize_link(other_uri_or_fs_path))
}

impl Item {
    pub fn has_link_to(&self, uri_or_fs_path: &str) -> bool {
        self.href()
            .map(|href| matches_link(href, uri_or_fs_path))
            .unwrap_or(false)
            || self
                .items()
                .map(|items| items.iter().any(|i| i.has_link_to(uri_or_fs_path)))
                .unwrap_or(false)
    }

    pub fn matches_href(&self, uri_or_fs_path: &str) -> bool {
        if let Some(href) = self.href() {
            matches_link(href, uri_or_fs_path)
        } else {
            false
        }
    }

    pub fn is_subheading(&self) -> bool {
        matches!(self, Item::Subheading { .. })
    }

    pub fn is_link(&self) -> bool {
        matches!(self, Item::Link { .. })
    }

    pub fn label(&self) -> &str {
        match self {
            Item::Link { label, .. } => label,
            Item::Subheading { label, .. } => label,
        }
    }

    pub fn href(&self) -> Option<&str> {
        match self {
            Item::Link { href, .. } => href.as_deref(),
            _ => None,
        }
    }

    pub fn external_href(&self) -> Option<&str> {
        match self {
            Item::Link { external_href, .. } => external_href.as_deref(),
            _ => None,
        }
    }

    pub fn heading(&self) -> Option<&str> {
        match self {
            Item::Subheading { label, .. } => Some(label),
            _ => None,
        }
    }

    pub fn items(&self) -> Option<&[Item]> {
        match &self {
            Item::Subheading { items, .. } => items.as_ref().map(|i| &i[..]),
            Item::Link { items, .. } => items.as_ref().map(|i| &i[..]),
        }
    }

    pub fn collapsible(&self) -> Option<bool> {
        match &self {
            Item::Subheading { collapsible, .. } => *collapsible,
            Item::Link { collapsible, .. } => *collapsible,
        }
    }

    pub fn collapsed(&self) -> Option<bool> {
        match &self {
            Item::Subheading { collapsed, .. } => *collapsed,
            Item::Link { collapsed, .. } => *collapsed,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum Filter {
    Equals { equals: String },
    OneOf { one_of: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct UserPreferencesFilter {
    pub user_preferences: HashMap<String, Filter>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SectionDescription {
    pub heading: Option<String>,
    pub collapsed: Option<bool>,
    pub collapsible: Option<bool>,
    pub items: Option<Vec<ItemDescription>>,
    pub show_if: Option<UserPreferencesFilter>,
}

impl SectionDescription {
    fn verify(&self, project: &Project) -> Vec<Error> {
        let mut errors = vec![];

        // User preference verifications
        if let Some(show_if) = &self.show_if {
            verify_user_preference_filter(show_if, project, &mut errors);
        }

        if let Some(items) = &self.items {
            for item in items {
                item.verify(project, &mut errors)
            }
        }

        errors
    }

    fn resolve(self, ctx: &RenderContext, project: &Project) -> Option<Section> {
        let SectionDescription {
            heading,
            collapsed,
            collapsible,
            items,
            show_if,
        } = self;

        let section = Section {
            heading,
            collapsed: collapsed.unwrap_or(false),
            collapsible: collapsible.or(collapsed).unwrap_or(false),
            items: items
                .map(|i| {
                    i.into_iter()
                        .filter_map(|i| i.resolve(ctx, project))
                        .flatten()
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default(),
        };

        if should_show(show_if.as_ref(), ctx) {
            Some(section)
        } else {
            None
        }
    }
}

/// Figure out if we should show a given item given its `show_if` filter and the
/// current render context.
///
/// In short, if we do have a filter based on a user preference, we check if the
/// filter matches either the provided user pref from the context, or the default
/// pref if one is not set by the user.
fn should_show(show_if: Option<&UserPreferencesFilter>, ctx: &RenderContext) -> bool {
    let filters = if let Some(s) = show_if {
        s
    } else {
        return true;
    };

    filters.user_preferences.iter().all(|(k, v)| match v {
        Filter::Equals { equals } => {
            ctx.options
                .user_preferences
                .get(k)
                .or_else(|| ctx.settings.user_preferences().get(k).map(|p| &p.default))
                == Some(equals)
        }
        Filter::OneOf { one_of } => one_of.iter().any(|l| {
            ctx.options
                .user_preferences
                .get(k)
                .or_else(|| ctx.settings.user_preferences().get(k).map(|p| &p.default))
                == Some(l)
        }),
    })
}

fn verify_user_preference_filter(
    filter: &UserPreferencesFilter,
    project: &Project,
    errors: &mut Vec<Error>,
) {
    verify_user_preference_filter_keys(filter, project, errors);
    verify_user_preference_filter_values(filter, project, errors);
}

fn verify_user_preference_filter_keys(
    filter: &UserPreferencesFilter,
    project: &Project,
    errors: &mut Vec<Error>,
) {
    for filter_key in filter.user_preferences.keys() {
        if !project
            .settings()
            .user_preferences()
            .keys()
            .any(|k| k == filter_key)
        {
            errors.push(Error {
                code: Error::INVALID_NAVIGATION,
                message: format!(
                    "Unknown user preference \"{}\" found in navigation",
                    filter_key
                ),
                description: if !project.settings().user_preferences().is_empty() {
                    format!(
                        "Expected one of [{}].\nFound \"{}\".",
                        project
                            .settings()
                            .user_preferences()
                            .keys()
                            .map(|s| format!("\"{}\"", s))
                            .collect::<Vec<_>>()
                            .join(", "),
                        filter_key
                    )
                } else {
                    "No custom user preferences defined in docapella.yaml.".to_string()
                },
                file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
                position: None,
            });
        }
    }
}

fn verify_user_preference_filter_values(
    filter: &UserPreferencesFilter,
    project: &Project,
    errors: &mut Vec<Error>,
) {
    for (key, filter) in filter.user_preferences.iter() {
        match filter {
            Filter::Equals { equals } => {
                if let Some(possible_values) = &project
                    .settings()
                    .user_preferences()
                    .get(key)
                    .map(|p| &p.values)
                {
                    if !possible_values.iter().any(|v| &v.value == equals) {
                        errors.push(Error {
                                code: Error::INVALID_NAVIGATION,
                                message: format!(
                                    "Unknown value \"{}\" for user preference \"{}\" found in navigation",
                                    equals,
                                    key
                                ),
                                description:  format!(
                                        "Expected one of [{}].\nFound \"{}\".",
                                        possible_values.iter()
                                            .map(|s| format!("\"{}\"", s))
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        equals
                                    )
                                ,
                                file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
            position: None,
                            });
                    }
                }
            }
            Filter::OneOf { ref one_of } => {
                // Can't be None because of above check
                if let Some(possible_values) = &project
                    .settings()
                    .user_preferences()
                    .get(key)
                    .map(|p| &p.values)
                {
                    for candidate in one_of {
                        if !possible_values.iter().any(|v| &v.value == candidate) {
                            errors.push(Error {
                                code: Error::INVALID_NAVIGATION,
                                message: format!(
                                    "Unknown value \"{}\" for user preference \"{}\" found in navigation",
                                    candidate,
                                    key
                                ),
                                description: format!(
                                        "Expected any of [{}].\nFound \"{}\".",
                                        possible_values.iter()
                                            .map(|s| format!("\"{}\"", s))
                                            .collect::<Vec<_>>()
                                            .join(", "),
                                        candidate
                                    )
                                ,
                                file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
            position: None,
                            });
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ItemDescription {
    Link {
        label: String,
        href: Option<String>,
        external: Option<String>,
        title: Option<String>,
        collapsed: Option<bool>,
        collapsible: Option<bool>,
        show_if: Option<UserPreferencesFilter>,
        items: Option<Vec<ItemDescription>>,
    },
    Subheading {
        subheading: String,
        collapsed: Option<bool>,
        collapsible: Option<bool>,
        show_if: Option<UserPreferencesFilter>,
        items: Option<Vec<ItemDescription>>,
    },
    OpenApi {
        open_api_spec: String,
        show_if: Option<UserPreferencesFilter>,
        only: Option<Vec<String>>,
    },
}

impl ItemDescription {
    fn verify(&self, project: &Project, errors: &mut Vec<Error>) {
        if let Some(show_if) = self.show_if() {
            verify_user_preference_filter(show_if, project, errors);
        }

        if let Some(items) = self.items() {
            for item in items {
                item.verify(project, errors)
            }
        }

        match &self {
            ItemDescription::Link { href, external, .. } => {
                if href.clone().or(external.clone()).is_none() {
                    errors.push(Error {
                        code: Error::NAVIGATION_ERROR,
                        message: "Invalid navigation.yaml".to_string(),
                        description: "Navigation items must have either href or external specified"
                            .to_string(),
                        file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
                        position: None,
                    })
                }

                if let Some(href) = href {
                    if markdown::parser::parse_external_link(href).is_some() {
                        errors.push(Error {
                            code: Error::NAVIGATION_ERROR,
                            message: "Invalid internal link in `href` found in navigation.yaml".to_string(),
                            description: format!("Found \"{}\", which is an external link. Use `external` instead of `href` for external urls", href),
                            file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
            position: None,
                        })
                    }
                }
            }
            ItemDescription::OpenApi {
                open_api_spec,
                only,
                ..
            } => {
                self.verify_open_api_spec_file_exists(open_api_spec.as_str(), project, errors);
                self.verify_open_api_only_filter_has_valid_tags(
                    open_api_spec.as_str(),
                    only.as_ref(),
                    project,
                    errors,
                );
            }
            _ => {}
        }
    }

    fn verify_open_api_spec_file_exists(
        &self,
        open_api_spec: &str,
        project: &Project,
        errors: &mut Vec<Error>,
    ) {
        if !project
            .pages()
            .iter()
            .any(|p| p.fs_path() == Path::new(&open_api_spec))
        {
            errors.push(Error {
                code: Error::NAVIGATION_ERROR,
                message: "Unknown OpenAPI spec file found in navigation".to_string(),
                description: format!(
                    "Expected one of [{}].\nFound \"{}\".",
                    project
                        .pages()
                        .iter()
                        .filter(|p| p.is_openapi())
                        .map(|p| format!("\"{}\"", p.fs_path().display()))
                        .collect::<HashSet<_>>()
                        .into_iter()
                        .collect::<Vec<_>>()
                        .join(", "),
                    open_api_spec
                ),
                file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
                position: None,
            });
        }
    }

    fn verify_open_api_only_filter_has_valid_tags(
        &self,
        open_api_spec: &str,
        only: Option<&Vec<String>>,
        project: &Project,
        errors: &mut Vec<Error>,
    ) {
        if let Some(tags) = only {
            for tag in tags {
                if !project.pages().iter().any(|p| {
                    p.fs_path() == Path::new(&open_api_spec)
                        && p.openapi_tag() == Some(tag.as_str())
                }) {
                    errors.push(Error {
                        code: Error::NAVIGATION_ERROR,
                        message: "Unknown OpenAPI tag found in navigation `only` filter"
                            .to_string(),
                        description: format!(
                            "Expected one of [{}].\nFound \"{}\".",
                            {
                                let mut all_tags = project
                                    .pages()
                                    .iter()
                                    .filter(|p| {
                                        p.is_openapi() && p.fs_path() == Path::new(&open_api_spec)
                                    })
                                    .map(|p| format!("\"{}\"", p.openapi_tag().unwrap()))
                                    .collect::<Vec<_>>();
                                all_tags.sort();
                                all_tags.join(", ")
                            },
                            tag
                        ),
                        file: Some(PathBuf::from(crate::NAVIGATION_FILE_NAME)),
                        position: None,
                    });
                }
            }
        }
    }

    fn resolve(self, ctx: &RenderContext, project: &Project) -> Option<Vec<Item>> {
        match self {
            ItemDescription::Subheading {
                subheading,
                collapsed,
                collapsible,
                items,
                show_if,
            } => {
                let heading = Item::Subheading {
                    label: subheading,
                    collapsed: Some(collapsed.unwrap_or(false)),
                    collapsible: collapsible.or(collapsed).or(Some(false)),
                    items: items.map(|s| {
                        s.into_iter()
                            .filter_map(|i| i.resolve(ctx, project))
                            .flatten()
                            .collect::<Vec<_>>()
                    }),
                };

                if should_show(show_if.as_ref(), ctx) {
                    Some(vec![heading])
                } else {
                    None
                }
            }
            ItemDescription::Link {
                label,
                href,
                external,
                title,
                collapsed,
                collapsible,
                items,
                show_if,
            } => {
                let link = Item::Link {
                    label,
                    href: href.map(|href| markdown::parser::to_final_link(&href, ctx)),
                    external_href: external,
                    title,
                    collapsed: Some(collapsed.unwrap_or(false)),
                    collapsible: collapsible.or(collapsed).or(Some(false)),
                    http_method: None,
                    items: items.map(|s| {
                        s.into_iter()
                            .filter_map(|i| i.resolve(ctx, project))
                            .flatten()
                            .collect::<Vec<_>>()
                    }),
                };

                if should_show(show_if.as_ref(), ctx) {
                    Some(vec![link])
                } else {
                    None
                }
            }
            ItemDescription::OpenApi {
                open_api_spec,
                show_if,
                only,
            } => {
                if should_show(show_if.as_ref(), ctx) {
                    Some(Self::resolve_open_api_specs(
                        open_api_spec.as_str(),
                        only.as_ref(),
                        project,
                        ctx,
                    ))
                } else {
                    None
                }
            }
        }
    }

    /// Builds the navigation structure for a given OpenAPI spec
    fn resolve_open_api_specs(
        open_api_spec_path: &str,
        only: Option<&Vec<String>>,
        project: &Project,
        ctx: &RenderContext,
    ) -> Vec<Item> {
        // Filter so that we only have the openapi specs
        // that are mentioned in the config
        if let Some(spec) = project
            .settings()
            .open_api()
            .iter()
            .find(|o| o.spec_file == Path::new(&open_api_spec_path))
        {
            let mut items = vec![];

            // First, push the overview page
            if only.is_none() {
                items.push(Item::Link {
                    label: "Overview".to_owned(),
                    title: Some("Overview".to_owned()),
                    href: Some(markdown::parser::to_final_link(&spec.uri_prefix, ctx)),
                    external_href: None,
                    http_method: None,
                    collapsed: Some(false),
                    collapsible: Some(false),
                    items: None,
                });
            }

            // Next, find the tag pages
            let mut pages = project
                .pages()
                .into_iter()
                .filter(|p| p.fs_path() == Path::new(&open_api_spec_path))
                .filter(|p| p.uri_path() != spec.uri_prefix) // Filter out the overview, since we only want tags
                .filter_map(|p| match &p.page {
                    PageKind::Markdown(_) => None,
                    PageKind::OpenApi(openapi_page) => Some(openapi_page),
                })
                .filter(|p| match (only, p.tag()) {
                    (None, None) => true,
                    (None, Some(_)) => true,
                    (Some(_), None) => false,
                    (Some(tags), Some(title)) => tags.contains(&title.to_owned()),
                })
                .collect::<Vec<_>>();

            // Sort by the `only` filter tag order, if we have a filter
            if let Some(tags) = only {
                pages.sort_by(|a, b| {
                    let a_title = a.tag().unwrap_or("Unknown tag");
                    let b_title = b.tag().unwrap_or("Unknown tag");

                    let a_pos = tags.iter().position(|t| t == a_title).unwrap_or(999);
                    let b_pos = tags.iter().position(|t| t == b_title).unwrap_or(999);

                    a_pos.cmp(&b_pos)
                });
            }

            // Create a link per page, with operations links as child items
            for page in pages {
                items.push(Item::Link {
                    label: page
                        .tag()
                        .map(|t| t.to_owned())
                        .unwrap_or_else(|| "Unknown tag".to_owned()),
                    href: Some(markdown::parser::to_final_link(&page.uri_path, ctx)),
                    external_href: None,
                    http_method: None,
                    title: page.tag().map(|t| t.to_owned()),
                    collapsed: Some(true),
                    collapsible: Some(true),
                    items: Some(
                        page.operations()
                            .iter()
                            .map(|op| Item::Link {
                                label: op.summary.as_ref().unwrap_or(&op.route_pattern).to_owned(),
                                title: Some(
                                    op.summary.as_ref().unwrap_or(&op.route_pattern).to_owned(),
                                ),
                                href: Some(markdown::parser::to_final_link(
                                    &format!("{}#{}", page.uri_path, op.anchor_tag),
                                    ctx,
                                )),
                                external_href: None,
                                http_method: HttpMethod::from_str(op.method.as_str()),
                                collapsed: Some(false),
                                collapsible: Some(false),
                                items: None,
                            })
                            .collect::<Vec<_>>(),
                    ),
                });
            }

            items
        } else {
            vec![]
        }
    }

    #[allow(dead_code)]
    fn is_subheading(&self) -> bool {
        matches!(self, ItemDescription::Subheading { .. })
    }

    #[allow(dead_code)]
    fn is_link(&self) -> bool {
        matches!(self, ItemDescription::Link { .. })
    }

    #[allow(dead_code)]
    fn is_open_api(&self) -> bool {
        matches!(self, ItemDescription::OpenApi { .. })
    }

    #[allow(dead_code)]
    fn label(&self) -> Option<&str> {
        match self {
            ItemDescription::Link { label, .. } => Some(label),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn href(&self) -> Option<&str> {
        match self {
            ItemDescription::Link { href, .. } => href.as_deref(),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn subheading(&self) -> Option<&str> {
        match self {
            ItemDescription::Subheading { subheading, .. } => Some(subheading),
            _ => None,
        }
    }

    #[allow(dead_code)]
    fn items(&self) -> Option<&[ItemDescription]> {
        match &self {
            ItemDescription::Subheading { items, .. } => items.as_ref().map(|i| &i[..]),
            ItemDescription::Link { items, .. } => items.as_ref().map(|i| &i[..]),
            ItemDescription::OpenApi { .. } => None,
        }
    }

    fn show_if(&self) -> Option<&UserPreferencesFilter> {
        match &self {
            ItemDescription::Subheading { show_if, .. } => show_if.as_ref(),
            ItemDescription::Link { show_if, .. } => show_if.as_ref(),
            ItemDescription::OpenApi { show_if, .. } => show_if.as_ref(),
        }
    }

    #[allow(dead_code)]
    fn collapsed(&self) -> Option<bool> {
        match &self {
            ItemDescription::Subheading { collapsed, .. } => *collapsed,
            ItemDescription::Link { collapsed, .. } => *collapsed,
            ItemDescription::OpenApi { .. } => None,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{InputContent, InputFile, RenderOptions};
    use pretty_assertions::assert_eq;

    const PET_STORE: &str = include_str!("../examples/open_api_specs/petstore.json");

    use super::*;

    #[derive(Debug)]
    struct ProjectBuilder {
        inputs: Vec<InputFile>,
    }

    impl ProjectBuilder {
        fn default() -> Self {
            ProjectBuilder {
                inputs: vec![
                    InputFile {
                        path: PathBuf::from("README.md"),
                        content: InputContent::Text(String::from("")),
                    },
                    InputFile {
                        path: PathBuf::from(crate::SETTINGS_FILE_NAME),
                        content: InputContent::Text(
                            indoc! {"
                            ---
                            title: An Project
                            "}
                            .to_owned(),
                        ),
                    },
                    InputFile {
                        path: PathBuf::from(crate::NAVIGATION_FILE_NAME),
                        content: InputContent::Text("---\n".to_owned()),
                    },
                ],
            }
        }

        fn with_file<P: Into<PathBuf>, S: Into<String>>(&mut self, path: P, content: S) {
            let path = path.into();
            let content = content.into();

            self.inputs
                .iter()
                .position(|i| i.path == path)
                .map(|pos| self.inputs.remove(pos));

            let file = InputFile {
                path,
                content: InputContent::Text(content),
            };

            self.inputs.push(file);
        }

        fn build(self) -> std::result::Result<Project, Vec<crate::Error>> {
            Project::from_file_list(self.inputs)
        }
    }

    #[test]
    fn description_basic() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Getting started"
              href: "/guides/getting-started"
            - label: "Advanced"
              href: "/guides/advanced"
        "#};

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("Guides"));
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0].label(),
            Some("Getting started")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0].href(),
            Some("/guides/getting-started")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[1].label(),
            Some("Advanced")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[1].href(),
            Some("/guides/advanced")
        );
    }

    #[test]
    fn description_without_heading() {
        let nav = indoc! {r#"
        - items:
            - label: "Getting started"
              href: "/guides/getting-started"
            - label: "Advanced"
              href: "/guides/advanced"
        "#};

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading, None);
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0].label(),
            Some("Getting started")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0].href(),
            Some("/guides/getting-started")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[1].label(),
            Some("Advanced")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[1].href(),
            Some("/guides/advanced")
        );
    }

    #[test]
    fn description_without_items() {
        let nav = indoc! {r#"
        - heading: "Guides"
        "#};

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("Guides"));
        assert_eq!(sections[0].items, None);
    }
    #[test]
    fn collapsed_section() {
        let nav = indoc! {r#"
        - heading: "Guides"
          collapsed: true
          collapsible: true
          items:
          - label: "Getting started"
            href: "/guides/getting-started"
        "#};

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].collapsed, Some(true));
        assert_eq!(sections[0].collapsible, Some(true));
    }

    #[test]
    fn description_nested_sections() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
          - label: "Getting started"
            href: "/guides/getting-started"
            items:
            - label: "Create an account"
              href: "/guides/getting-started/create-an-account"
            - label: "First resource"
              href: "/guides/getting-started/first-resource"
        "#};

        let sections = parse_description(nav).unwrap();

        assert_eq!(
            sections[0].items.as_ref().unwrap()[0].items().unwrap()[0].label(),
            Some("Create an account")
        );
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0].items().unwrap()[0].href(),
            Some("/guides/getting-started/create-an-account")
        );
    }

    #[test]
    fn resolve_nested_sections() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
          - subheading: "Getting started"
            items:
            - label: "Create an account"
              href: "/guides/getting-started/create-an-account"
            - label: "First resource"
              href: "/guides/getting-started/first-resource"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("Guides"));
        assert!(sections[0].items[0].is_subheading());
        assert_eq!(sections[0].items[0].label(), "Getting started");
        assert!(sections[0].items[0].items().unwrap()[0].is_link());
        assert!(sections[0].items[0].items().unwrap()[1].is_link());
    }

    #[test]
    fn webbifys_internal_urls() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
          - label: "First resource"
            href: "/guides/getting-started/first-resource.md"
        "#};

        let opts = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &ctx, &project).unwrap();

        assert_eq!(
            sections[0].items[0].href(),
            Some("/guides/getting-started/create-an-account")
        );
        assert_eq!(
            sections[0].items[1].href(),
            Some("/guides/getting-started/first-resource")
        );
    }

    #[test]
    fn webbifys_internal_urls_with_anchor() {
        let nav = indoc! {r#"
        - heading: "Test"
          items:
          - label: "Foo"
            href: "README.md#foo"
          - label: "Bar"
            href: "bar.md#bar"
        "#};

        let opts = RenderOptions {
            webbify_internal_urls: true,
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file("bar.md", "");
        let project = builder.build().unwrap();

        assert!(
            project.verify(None, None).is_ok(),
            "Project failed verification: {:#?}",
            project.verify(None, None)
        );

        let sections = build(nav, &ctx, &project).unwrap();

        assert_eq!(sections[0].items[0].href(), Some("/#foo"));
        assert_eq!(sections[0].items[1].href(), Some("/bar#bar"));
    }

    #[test]
    fn set_url_prefixes() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
          - label: "First resource"
            href: "/guides/getting-started/first-resource.md"
        "#};

        let opts = RenderOptions {
            prefix_link_urls: Some("/foo".to_string()),
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &ctx, &project).unwrap();

        assert_eq!(
            sections[0].items[0].href(),
            Some("/foo/guides/getting-started/create-an-account.md")
        );
        assert_eq!(
            sections[0].items[1].href(),
            Some("/foo/guides/getting-started/first-resource.md")
        );
    }

    #[test]
    fn set_url_prefixes_and_webbify() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
          - label: "First resource"
            href: "/guides/getting-started/first-resource.md"
        "#};

        let opts = RenderOptions {
            prefix_link_urls: Some("/foo".to_string()),
            webbify_internal_urls: true,
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &ctx, &project).unwrap();

        assert_eq!(
            sections[0].items[0].href(),
            Some("/foo/guides/getting-started/create-an-account")
        );
        assert_eq!(
            sections[0].items[1].href(),
            Some("/foo/guides/getting-started/first-resource")
        );
    }

    #[test]
    fn default_non_collapsed() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(!sections[0].collapsed);
        assert_eq!(sections[0].items[0].collapsed(), Some(false));
        assert_eq!(sections[0].items[0].collapsible(), Some(false));
    }

    #[test]
    fn collapsed_implies_collapsible() {
        let nav = indoc! {r#"
        - heading: "Guides"
          collapsed: true
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
            collapsed: true
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
          - subheading: "Create an account"
            collapsed: true
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(sections[0].collapsed);
        assert!(sections[0].collapsible);
        assert_eq!(sections[0].items[0].collapsed(), Some(true));
        assert_eq!(sections[0].items[0].collapsible(), Some(true));
        assert_eq!(sections[0].items[1].collapsed(), Some(true));
        assert_eq!(sections[0].items[1].collapsible(), Some(true));
    }

    #[test]
    fn collapsible_does_not_imply_collapsed() {
        let nav = indoc! {r#"
        - heading: "Guides"
          collapsible: true
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
            collapsible: true
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
          - subheading: "Create an account"
            collapsible: true
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(!sections[0].collapsed);
        assert!(sections[0].collapsible);
        assert_eq!(sections[0].items[0].collapsed(), Some(false));
        assert_eq!(sections[0].items[0].collapsible(), Some(true));
        assert_eq!(sections[0].items[1].collapsed(), Some(false));
        assert_eq!(sections[0].items[1].collapsible(), Some(true));
    }

    #[test]
    fn can_be_collapsed_but_not_collapsible() {
        // Collapsible will override in the UI
        let nav = indoc! {r#"
        - heading: "Guides"
          collapsible: false
          collapsed: true
          items:
          - label: "Create an account"
            href: "/guides/getting-started/create-an-account.md"
            collapsible: false
            collapsed: true
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
          - subheading: "Create an account"
            collapsible: false
            collapsed: true
            items:
            - label: "First resource"
              href: "/guides/getting-started/first-resource.md"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let sections = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(sections[0].collapsed);
        assert!(!sections[0].collapsible);
        assert_eq!(sections[0].items[0].collapsed(), Some(true));
        assert_eq!(sections[0].items[0].collapsible(), Some(false));
        assert_eq!(sections[0].items[1].collapsed(), Some(true));
        assert_eq!(sections[0].items[1].collapsible(), Some(false));
    }

    #[test]
    fn can_mark_sections_to_be_shown_given_a_user_preference() {
        let nav = indoc! {r#"
        - heading: "Guides"
          show_if:
            user_preferences:
              plan:
                equals: Plan A
          items:
            - label: "Create an account"
              href: "/guides/getting-started/create-an-account.md"
        "#};
        let settings = indoc! {r#"
        ---
        title: Example
        user_preferences:
          plan:
            label: Plan
            default: Plan A
            values:
              - Plan A
              - Plan B
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        let project = builder.build().unwrap();

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan A".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections.len(), 1);

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan B".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections.len(), 0);
    }

    #[test]
    fn can_mark_links_to_be_shown_given_a_user_preference() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Create an account"
              href: "/guides/getting-started/create-an-account.md"
              show_if:
                user_preferences:
                  plan:
                    equals: Plan A
        "#};
        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);

        let project = builder.build().unwrap();

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan A".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections[0].items.len(), 1);

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan B".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections[0].items.len(), 0);
    }

    #[test]
    fn verify_nested_show_if_user_preferences() {
        let settings = indoc! {r#"
        ---
        title: Example
        user_preferences:
          plan:
            label: Plan
            default: Plan A
            values:
              - Plan A
              - Plan B
        "#};

        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Create an account"
              href: README.md
              show_if:
                user_preferences:
                  plan:
                    equals: SURPRISE BATMAN
              items:
              - label: "Managing an account"
                href: README.md
                show_if:
                  user_preferences:
                    plan:
                      equals: SURPRISE SPIDERMAN
        "#};
        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Bad user preferences in nav not found");

        assert_eq!(errors.len(), 2);
    }

    #[test]
    fn verify_items_have_href_or_external() {
        let settings = indoc! {r#"
        ---
        title: Example
        "#};

        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Create an account"
        "#};
        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Navigation items must have either href or external specified");

        assert_eq!(errors.first().unwrap().message, "Invalid navigation.yaml");
        assert_eq!(
            errors.first().unwrap().description,
            "Navigation items must have either href or external specified"
        );

        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn verify_href_is_internal() {
        let settings = indoc! {r#"
        ---
        title: Example
        "#};

        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Create an account"
              href: https://www.example.com
        "#};
        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err(r#"Use the "external" property for external urls"#);

        assert_eq!(
            errors.first().unwrap().message,
            "Invalid internal link in `href` found in navigation.yaml"
        );
        assert_eq!(
            errors.first().unwrap().description,
            "Found \"https://www.example.com\", which is an external link. Use `external` instead of `href` for external urls"
        );

        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn can_mark_subheadings_to_be_shown_given_a_user_preference() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - subheading: "Create an account"
              show_if:
                user_preferences:
                  plan:
                    equals: Plan A
        "#};
        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan A".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections[0].items.len(), 1);

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan B".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections[0].items.len(), 0);
    }

    #[test]
    fn can_mark_sections_to_be_shown_if_a_user_preference_matches_any_in_a_list() {
        let nav = indoc! {r#"
        - heading: "Guides"
          show_if:
            user_preferences:
              plan:
                one_of:
                  - Plan A
                  - Plan B
          items:
            - label: "Create an account"
              href: "/guides/getting-started/create-an-account.md"
        "#};

        let settings = indoc! {r#"
        ---
        title: Example
        user_preferences:
          plan:
            label: Plan
            default: Plan A
            values:
              - Plan A
              - Plan B
              - Plan C
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        let project = builder.build().unwrap();

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan A".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections.len(), 1);

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan B".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections.len(), 1);

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan C".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections.len(), 0);
    }

    #[test]
    fn tells_you_if_a_path_is_contained_in_the_nav_structure() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Getting started"
              href: "/guides/getting-started"
            - label: "Advanced"
              href: "/guides/advanced"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(navigation.has_link_to("/guides/advanced"));
        assert!(!navigation.has_link_to("/guides/not-advanced"));
    }

    #[test]
    fn tells_you_if_a_path_is_contained_in_the_nested_nav_structure() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Getting started"
              href: "/guides/getting-started"
              items:
                - label: "Advanced"
                  href: "/guides/advanced"
            - subheading: Subheading
              items:
                - label: "Advanced"
                  href: "/guides/needle"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(navigation.has_link_to("/guides/advanced"));
        assert!(navigation.has_link_to("/guides/needle"));
    }

    #[test]
    fn tells_you_if_a_path_is_contained_in_the_nested_nav_structure_even_if_missing_prefix_slash() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Getting started"
              href: "Getting-Started.md"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(navigation.has_link_to("/Getting-Started.md"));
        assert!(navigation.has_link_to("Getting-Started.md"));
    }

    #[test]
    fn always_says_has_link_to_root_url() {
        let nav = indoc! {r#"
        - heading: "Guides"
          items:
            - label: "Getting started"
              href: "Getting-Started.md"
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(navigation.has_link_to("/"));
    }

    #[test]
    fn openapi_basic_navigation() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.json
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.json
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file("openapi.json", PET_STORE);
        let project = builder.build().unwrap();

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("API"));
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0],
            ItemDescription::OpenApi {
                open_api_spec: "openapi.json".to_string(),
                show_if: None,
                only: None,
            }
        );

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(
            navigation.sections[0].items,
            vec![
                Item::Link {
                    label: "Overview".to_owned(),
                    title: Some("Overview".to_owned()),
                    href: Some("/api".to_owned()),
                    external_href: None,
                    http_method: None,
                    collapsible: Some(false),
                    collapsed: Some(false),
                    items: None,
                },
                Item::Link {
                    label: "pets".to_owned(),
                    title: Some("pets".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/api/pets".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![
                        Item::Link {
                            label: "List all pets".to_owned(),
                            title: Some("List all pets".to_owned()),
                            href: Some("/api/pets#list-all-pets".to_owned()),
                            external_href: None,
                            http_method: Some(HttpMethod::Get),
                            collapsible: Some(false),
                            collapsed: Some(false),
                            items: None,
                        },
                        Item::Link {
                            label: "Create a pet".to_owned(),
                            title: Some("Create a pet".to_owned()),
                            href: Some("/api/pets#create-a-pet".to_owned()),
                            external_href: None,
                            http_method: Some(HttpMethod::Post),
                            collapsible: Some(false),
                            collapsed: Some(false),
                            items: None,
                        },
                        Item::Link {
                            label: "Info for a specific pet".to_owned(),
                            title: Some("Info for a specific pet".to_owned()),
                            href: Some("/api/pets#info-for-a-specific-pet".to_owned()),
                            external_href: None,
                            http_method: Some(HttpMethod::Get),
                            collapsible: Some(false),
                            collapsed: Some(false),
                            items: None,
                        },
                    ]),
                }
            ]
        );
    }

    #[test]
    fn openapi_basic_navigation_with_only_filter() {
        use pretty_assertions::assert_eq;

        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
              only:
                - puppies
                - bunnies
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_explicit.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("API"));
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0],
            ItemDescription::OpenApi {
                open_api_spec: "openapi.yaml".to_string(),
                show_if: None,
                only: Some(vec!["puppies".to_owned(), "bunnies".to_owned()])
            }
        );

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(
            navigation.sections[0].items,
            vec![
                Item::Link {
                    label: "puppies".to_owned(),
                    title: Some("puppies".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/api/puppies".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![Item::Link {
                        label: "Create a pet".to_owned(),
                        title: Some("Create a pet".to_owned()),
                        href: Some("/api/puppies#create-a-pet".to_owned()),
                        external_href: None,
                        http_method: Some(HttpMethod::Post),
                        collapsible: Some(false),
                        collapsed: Some(false),
                        items: None,
                    },]),
                },
                Item::Link {
                    label: "bunnies".to_owned(),
                    title: Some("bunnies".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/api/bunnies".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![Item::Link {
                        label: "Info for a specific pet".to_owned(),
                        title: Some("Info for a specific pet".to_owned()),
                        href: Some("/api/bunnies#info-for-a-specific-pet".to_owned()),
                        external_href: None,
                        http_method: Some(HttpMethod::Get),
                        collapsible: Some(false),
                        collapsed: Some(false),
                        items: None,
                    },]),
                }
            ]
        );
    }

    #[test]
    fn openapi_basic_navigation_with_only_filter_determines_order() {
        use pretty_assertions::assert_eq;

        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
              only:
                - kittens
                - puppies
                - bunnies
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_explicit.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("API"));
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0],
            ItemDescription::OpenApi {
                open_api_spec: "openapi.yaml".to_string(),
                show_if: None,
                only: Some(vec![
                    "kittens".to_owned(),
                    "puppies".to_owned(),
                    "bunnies".to_owned()
                ])
            }
        );

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(
            navigation.sections[0].items,
            vec![
                Item::Link {
                    label: "kittens".to_owned(),
                    title: Some("kittens".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/api/kittens".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![Item::Link {
                        label: "List all pets".to_owned(),
                        title: Some("List all pets".to_owned()),
                        href: Some("/api/kittens#list-all-pets".to_owned()),
                        http_method: Some(HttpMethod::Get),
                        external_href: None,
                        collapsible: Some(false),
                        collapsed: Some(false),
                        items: None,
                    },]),
                },
                Item::Link {
                    label: "puppies".to_owned(),
                    title: Some("puppies".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/api/puppies".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![Item::Link {
                        label: "Create a pet".to_owned(),
                        title: Some("Create a pet".to_owned()),
                        href: Some("/api/puppies#create-a-pet".to_owned()),
                        external_href: None,
                        http_method: Some(HttpMethod::Post),
                        collapsible: Some(false),
                        collapsed: Some(false),
                        items: None,
                    },]),
                },
                Item::Link {
                    label: "bunnies".to_owned(),
                    title: Some("bunnies".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/api/bunnies".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![Item::Link {
                        label: "Info for a specific pet".to_owned(),
                        title: Some("Info for a specific pet".to_owned()),
                        href: Some("/api/bunnies#info-for-a-specific-pet".to_owned()),
                        external_href: None,
                        http_method: Some(HttpMethod::Get),
                        collapsible: Some(false),
                        collapsed: Some(false),
                        items: None,
                    },]),
                },
            ]
        );
    }

    #[test]
    fn openapi_basic_navigation_warns_if_a_tag_mentioned_in_only_filter_does_not_exist() {
        use pretty_assertions::assert_eq;

        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
              only:
                - kittens
                - DRAGONS
                - UNICORNS
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_explicit.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let sections = parse_description(nav).unwrap();

        assert_eq!(sections[0].heading.as_deref(), Some("API"));
        assert_eq!(
            sections[0].items.as_ref().unwrap()[0],
            ItemDescription::OpenApi {
                open_api_spec: "openapi.yaml".to_string(),
                show_if: None,
                only: Some(vec![
                    "kittens".to_owned(),
                    "DRAGONS".to_owned(),
                    "UNICORNS".to_owned()
                ])
            }
        );

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(
            navigation.sections[0].items,
            vec![Item::Link {
                label: "kittens".to_owned(),
                title: Some("kittens".to_owned()),
                collapsible: Some(true),
                collapsed: Some(true),
                href: Some("/api/kittens".to_owned()),
                external_href: None,
                http_method: None,
                items: Some(vec![Item::Link {
                    label: "List all pets".to_owned(),
                    title: Some("List all pets".to_owned()),
                    href: Some("/api/kittens#list-all-pets".to_owned()),
                    external_href: None,
                    http_method: Some(HttpMethod::Get),
                    collapsible: Some(false),
                    collapsed: Some(false),
                    items: None,
                },]),
            }]
        );

        let errors = project
            .verify(None, None)
            .expect_err("Bad `only` tag filter in nav not found");

        assert_eq!(errors.len(), 2);

        assert_eq!(
            errors[0].message,
            "Unknown OpenAPI tag found in navigation `only` filter"
        );
        assert_eq!(
            errors[1].message,
            "Unknown OpenAPI tag found in navigation `only` filter"
        );

        assert_eq!(
            errors[0].description,
            "Expected one of [\"bunnies\", \"kittens\", \"puppies\"].\nFound \"DRAGONS\"."
        );
        assert_eq!(
            errors[1].description,
            "Expected one of [\"bunnies\", \"kittens\", \"puppies\"].\nFound \"UNICORNS\"."
        );
    }

    #[test]
    fn can_handle_open_api_anchor_tags_when_checking_link_existence() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.json
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.json
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file("openapi.json", PET_STORE);
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert!(navigation.has_link_to("/api/pets#create-a-pet"));
    }

    #[test]
    fn openapi_tag_ordering_with_explicit_tags() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_explicit.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(navigation.sections[0].items[1].label(), "puppies");
        assert_eq!(navigation.sections[0].items[2].label(), "bunnies");
        assert_eq!(navigation.sections[0].items[3].label(), "kittens");
    }

    #[test]
    fn openapi_tag_ordering_with_extension_tags() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_extension.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(navigation.sections[0].items.len(), 4);
        assert_eq!(navigation.sections[0].items[0].label(), "Overview");
        assert_eq!(navigation.sections[0].items[1].label(), "first_tag");
        assert_eq!(navigation.sections[0].items[2].label(), "extension_tag");
        assert_eq!(navigation.sections[0].items[3].label(), "second_tag");
    }

    #[test]
    fn openapi_tag_ordering_with_inline_tags() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_inline.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let navigation = build(nav, &RenderContext::new(), &project).unwrap();

        assert_eq!(navigation.sections[0].items[1].label(), "kittens");
        assert_eq!(navigation.sections[0].items[2].label(), "puppies");
        assert_eq!(navigation.sections[0].items[3].label(), "bunnies");
    }

    #[test]
    fn openapi_navigation_passes_verification() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_inline.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        assert!(
            project.verify(None, None).is_ok(),
            "Project failed verification: {:#?}",
            project.verify(None, None)
        );
    }

    #[test]
    fn openapi_navigation_works_with_uri_prefixes() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_inline.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let opts = RenderOptions {
            prefix_link_urls: Some("/_preview".to_string()),
            ..Default::default()
        };

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let navigation = build(nav, &ctx, &project).unwrap();

        assert!(
            navigation.sections[0].items[1]
                .href()
                .expect("No href?")
                .starts_with("/_preview"),
            "URL prefix not applied to openapi link"
        );

        assert!(
            navigation.sections[0].items[1].items().unwrap()[0]
                .href()
                .expect("No href?")
                .starts_with("/_preview"),
            "URL prefix not applied to openapi link"
        );
    }

    #[test]
    fn openapi_verifies_the_open_api_file_exists() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: nope.yaml
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_inline.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Project was valid with incorrect openapi in nav");

        assert_eq!(
            &errors[0].message,
            "Unknown OpenAPI spec file found in navigation"
        );
        assert_eq!(
            &errors[0].description,
            "Expected one of [\"openapi.yaml\"].\nFound \"nope.yaml\"."
        );
    }

    #[test]
    fn openapi_can_be_hidden_based_on_user_preferences() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.yaml
              show_if:
                user_preferences:
                  plan:
                    equals: Plan B
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example

        user_preferences:
          plan:
            label: Plan
            default: Plan A
            values:
              - Plan A
              - Plan B

        open_api:
            - spec_file: openapi.yaml
              uri_prefix: /api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file(
            "openapi.yaml",
            std::fs::read_to_string("./examples/open_api_specs/tag_order_inline.yaml").unwrap(),
        );
        let project = builder.build().unwrap();

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan A".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections[0].items.len(), 0);

        let mut opts = RenderOptions::default();
        opts.user_preferences
            .insert("plan".to_string(), "Plan B".to_string());

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);

        let sections = build(nav, &ctx, &project).unwrap();
        assert_eq!(sections[0].items.len(), 4);
    }

    #[test]
    fn openapi_does_not_prefix_navigation_urls_with_subtab_path() {
        let nav = indoc! {r#"
        - heading: API
          items:
            - open_api_spec: openapi.json
        "#};

        let settings = indoc! {r#"
        ---
        title: OpenAPI Example
        open_api:
            - spec_file: openapi.json
              uri_prefix: /tab1/api
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        builder.with_file("openapi.json", PET_STORE);
        let project = builder.build().unwrap();
        let opts = RenderOptions::default();

        let mut ctx = RenderContext::new();
        ctx.with_options(&opts);
        let navigation = build(nav, &ctx, &project).unwrap();

        assert_eq!(
            navigation.sections[0].items,
            vec![
                Item::Link {
                    label: "Overview".to_owned(),
                    title: Some("Overview".to_owned()),
                    href: Some("/tab1/api".to_owned()),
                    external_href: None,
                    http_method: None,
                    collapsible: Some(false),
                    collapsed: Some(false),
                    items: None,
                },
                Item::Link {
                    label: "pets".to_owned(),
                    title: Some("pets".to_owned()),
                    collapsible: Some(true),
                    collapsed: Some(true),
                    href: Some("/tab1/api/pets".to_owned()),
                    external_href: None,
                    http_method: None,
                    items: Some(vec![
                        Item::Link {
                            label: "List all pets".to_owned(),
                            title: Some("List all pets".to_owned()),
                            href: Some("/tab1/api/pets#list-all-pets".to_owned()),
                            external_href: None,
                            http_method: Some(HttpMethod::Get),
                            collapsible: Some(false),
                            collapsed: Some(false),
                            items: None,
                        },
                        Item::Link {
                            label: "Create a pet".to_owned(),
                            title: Some("Create a pet".to_owned()),
                            href: Some("/tab1/api/pets#create-a-pet".to_owned()),
                            external_href: None,
                            http_method: Some(HttpMethod::Post),
                            collapsible: Some(false),
                            collapsed: Some(false),
                            items: None,
                        },
                        Item::Link {
                            label: "Info for a specific pet".to_owned(),
                            title: Some("Info for a specific pet".to_owned()),
                            href: Some("/tab1/api/pets#info-for-a-specific-pet".to_owned()),
                            external_href: None,
                            http_method: Some(HttpMethod::Get),
                            collapsible: Some(false),
                            collapsed: Some(false),
                            items: None,
                        },
                    ]),
                }
            ]
        );
    }

    #[test]
    fn prefixes_absolute_navigation_urls() {
        let tab_nav = indoc! {r#"
        - heading: Section1
          items:
            - label: Foo
              href: /tab1/foo.md
            - label: Bar
              href: /bar.md
        "#};

        let settings = indoc! {r#"
        ---
        title: Sections Example
        tabs:
          - label: Tab1
            path: /tab1/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(format!("tab1/{}", crate::NAVIGATION_FILE_NAME), tab_nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        let project = builder.build().unwrap();
        let opts = RenderOptions {
            prefix_link_urls: Some("/v1".to_string()),
            ..Default::default()
        };

        let nav = project.navigation(Some(&opts), "/tab1/").unwrap();

        assert_eq!(nav.sections[0].items[0].href(), Some("/v1/tab1/foo.md"));
        assert_eq!(nav.sections[0].items[1].href(), Some("/v1/bar.md"));
    }
    #[test]
    fn prefixs_and_expands_relative_navigation_urls() {
        let tab_nav = indoc! {r#"
        - heading: Section1
          items:
            - label: Foo
              href: ./foo.md
            - label: Bar
              href: ../bar.md
            - label: Baz
              href: Baz Baz.md
        "#};

        let settings = indoc! {r#"
        ---
        title: Sections Example
        tabs:
          - label: Tab1
            path: /tab1/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(format!("tab1/{}", crate::NAVIGATION_FILE_NAME), tab_nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        let project = builder.build().unwrap();
        let opts = RenderOptions {
            prefix_link_urls: Some("/v1".to_string()),
            ..Default::default()
        };

        let nav = project.navigation(Some(&opts), "/tab1/").unwrap();

        assert_eq!(nav.sections[0].items[0].href(), Some("/v1/tab1/foo.md"));
        assert_eq!(nav.sections[0].items[1].href(), Some("/v1/bar.md"));
        assert_eq!(nav.sections[0].items[2].href(), Some("/v1/tab1/Baz Baz.md"));
    }

    #[test]
    fn does_not_prefix_external_hrefs() {
        let nav = indoc! {r#"
        - heading: Section1
          items:
            - label: Foo
              external: https://www.example.com
        "#};

        let settings = indoc! {r#"
        ---
        title: Sections Example
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);
        let project = builder.build().unwrap();
        let opts = RenderOptions {
            prefix_link_urls: Some("/v1".to_string()),
            ..Default::default()
        };

        let nav = project.navigation(Some(&opts), "/").unwrap();

        assert_eq!(
            nav.sections[0].items[0].external_href(),
            Some("https://www.example.com")
        );
    }

    #[test]
    fn gather_links_only_internal() {
        let settings = indoc! {r#"
        ---
        title: Example
        "#};

        let nav = indoc! {r#"
        - heading: Guides
          items:
            - label: Some Link
              href: /some-link.md
            - label: Other Link
              href: /some-link.md#foo
            - label: Example
              href: https://www.example.com
        "#};
        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::NAVIGATION_FILE_NAME, nav);
        builder.with_file(crate::SETTINGS_FILE_NAME, settings);

        let project = builder.build().unwrap();

        let nav = project
            .root_navigation(Some(&RenderOptions::default()))
            .unwrap();

        let links = nav.gather_links();

        assert_eq!(
            links,
            vec!["/some-link.md".to_string(), "/some-link.md#foo".to_string()]
        );
        assert_eq!(links.len(), 2);
    }
}
