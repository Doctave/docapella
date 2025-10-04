use regex::Regex;

use crate::parser::{is_external_link, rewrite_image_src, to_final_link};
use crate::project::Asset;
use crate::render_context::RenderContext;
use crate::tabs::{TabDescription, TabsList};
/// Settings for a given site backed by a `docapella.yaml` file.
use crate::{Error, Project, RenderOptions, Result, SETTINGS_FILE_NAME};
/// Settings for a given site backed by a `docapella.yaml` file.
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::{self, Display};
use std::path::{Path, PathBuf};
use url::Url;

lazy_static! {
    /// Check if a wildcard "*" or "**" is the end of the string
    /// Checks that string doesn't have more than one wildcard
    /// /foo/*/**  -> FAIL
    /// /foo/*/bar -> FAIL
    /// /foo/bar/* -> PASS
    /// /*         -> PASS
    static ref WILDCARD_REGEX: Regex = Regex::new(r"(?:[^\*]|^)\/\*{1,2}$").unwrap();

    /// Check if path parameters contains non-alphanumerics
    /// /foo/:bar        -> PASS
    /// /foo/:bar/some   -> PASS
    /// /foo/:bar.f      -> FAIL
    /// /foo/:bar.f/some -> FAIL
    /// IF YOU CHANGE THIS, MAKE SURE TO UPDATE IN VENUE
    static ref PATH_REGEX: Regex = Regex::new(r"(:[\wöäå]+)(?:\/|$)").unwrap();
}

impl Settings {
    /// Unforunately it is impossible at the moment to ergonomically have an Enum in serde where
    /// one of the variants would be "untagged". In `VersionedSettings` the variants mark
    /// themselves with a `version: N` field, but we also need to be backwards compatible and
    /// support not having a version field at all. This is apparently hard.
    ///
    /// Read up on this thread:
    /// https://github.com/serde-rs/serde/issues/912
    ///
    /// Specifically this comment seemed to describe our exact issue:
    /// https://github.com/serde-rs/serde/issues/912#issuecomment-423643892
    ///
    pub fn parse(input: &str) -> Result<Self> {
        serde_yaml::from_str::<Settings>(input).map_err(|e| Error {
            code: Error::INVALID_DOCTAVE_YAML,
            message: "Invalid docapella.yaml".to_owned(),
            description: format!("There was an error parsing your docapella.yaml:\n\n{}", e),
            file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
            position: None,
        })
    }

    /// Rewrite links based on a list of link rewrites
    pub(crate) fn rewrite_links(&mut self, opts: &RenderOptions, assets: &[Asset]) {
        if let Some(logo) = self.theme.logo.as_mut() {
            logo.src = PathBuf::from(rewrite_image_src(
                &format!(
                    "/{}",
                    logo.src.display().to_string().trim_start_matches('/')
                ),
                opts,
                assets,
            ));
            logo.src_dark = logo.src_dark.as_mut().map(|dark_logo| {
                PathBuf::from(rewrite_image_src(
                    &format!(
                        "/{}",
                        dark_logo.display().to_string().trim_start_matches('/'),
                    ),
                    opts,
                    assets,
                ))
            });
        }

        if let Some(favicon) = self.theme.favicon.as_mut() {
            favicon.src = PathBuf::from(rewrite_image_src(
                &format!(
                    "/{}",
                    favicon.src.display().to_string().trim_start_matches('/')
                ),
                opts,
                assets,
            ));
        }

        if let Some(header) = self.header.as_mut() {
            if let Some(HeaderLink::Internal(link)) = header.cta.as_mut() {
                if link.download {
                    link.href = rewrite_image_src(
                        &format!("/{}", link.href.trim_start_matches('/')),
                        opts,
                        assets,
                    );
                }
            }

            for link in header.links.iter_mut() {
                if let HeaderLink::Internal(link) = link {
                    if link.download {
                        link.href = rewrite_image_src(
                            &format!("/{}", link.href.trim_start_matches('/')),
                            opts,
                            assets,
                        );
                    }
                }
            }
        }

        let footer = &mut self.footer;
        for link in footer.links.iter_mut() {
            if let HeaderLink::Internal(link) = link {
                if link.download {
                    link.href = rewrite_image_src(
                        &format!("/{}", link.href.trim_start_matches('/')),
                        opts,
                        assets,
                    );
                }
            }
        }
    }

    /// Resolve internal paths in settings
    pub(crate) fn resolve_paths(&mut self, opts: &RenderOptions, fs_path: &Path) {
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(Some(opts));
        ctx.with_url_base_by_fs_path(fs_path);

        if let Some(header) = self.header.as_mut() {
            if let Some(HeaderLink::Internal(link)) = header.cta.as_mut() {
                if !link.download {
                    link.href = to_final_link(&link.href, &ctx);
                }
            }

            for link in header.links.iter_mut() {
                if let HeaderLink::Internal(link) = link {
                    if !link.download {
                        link.href = to_final_link(&link.href, &ctx);
                    }
                }
            }
        }

        let footer = &mut self.footer;
        for link in footer.links.iter_mut() {
            if let HeaderLink::Internal(link) = link {
                if !link.download {
                    link.href = to_final_link(&link.href, &ctx);
                }
            }
        }
    }

    pub fn tabs(&self) -> Option<TabsList> {
        if self.tab_descriptions.is_empty() {
            None
        } else {
            Some(TabsList::from_tab_descriptions(
                self.tab_descriptions.clone(),
            ))
        }
    }

    pub fn title(&self) -> &str {
        self.title.as_str()
    }

    pub fn header(&self) -> Option<&HeaderSettings> {
        self.header.as_ref()
    }

    pub fn logo(&self) -> Option<&Logo> {
        self.theme.logo.as_ref()
    }

    pub fn favicon(&self) -> Option<&Favicon> {
        self.theme.favicon.as_ref()
    }

    pub fn theme(&self) -> Option<&Theme> {
        Some(&self.theme)
    }

    pub fn open_api(&self) -> &[OpenApi] {
        self.open_api.as_slice()
    }

    pub fn footer(&self) -> Option<&Footer> {
        Some(&self.footer)
    }

    pub fn redirects(&self) -> &[Redirect] {
        self.redirects.as_slice()
    }

    pub fn vale(&self) -> Option<&ValeSettings> {
        self.vale.as_ref()
    }

    pub fn styles(&self) -> &[PathBuf] {
        self.styles.as_slice()
    }

    pub fn verify(&self, project: &Project, errors: &mut Vec<Error>) {
        // Shared verifications
        self.verify_openapi_specs(project, errors);
        self.verify_favicon(project, errors);
        self.verify_styles(project, errors);
        self.verify_redirects(project, errors);
        self.verify_logo(project, errors);
        self.verify_header(project, errors);
        self.verify_footer(project, errors);
        self.verify_vale(project, errors);

        // Theme verifications
        self.verify_v2_theme(errors);
    }

    pub fn verify_vale(&self, project: &Project, errors: &mut Vec<Error>) {
        if let Some(config_path) = self.vale().and_then(|v| v.config_file_path.as_ref()) {
            let p = PathBuf::from(config_path);

            if let Some(file_name) = p.file_name().and_then(|file_name| file_name.to_str()) {
                if file_name.starts_with('.') && file_name != ".vale.ini" {
                    errors.push(Error {
                        code: Error::VALE_ERROR,
                        message: String::from("A hidden vale configuration file must be named \".vale.ini\"."),
                        description: format!("Use \".vale.ini\", or remove the \".\" from the start of the config file name \"{}\".", file_name),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }
            }

            if !project.input_paths.iter().any(|f| f == &p) {
                errors.push(Error {
                    code: Error::VALE_ERROR,
                    message: String::from("Couldn't find Vale configuration file."),
                    description: format!("Expected a Vale configuration file at \"{config_path}\""),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                })
            }
        }
    }

    fn verify_header(&self, project: &Project, errors: &mut Vec<Error>) {
        if let Some(header) = self.header() {
            if let Some(cta) = &header.cta {
                cta.verify(project, "header", errors);
            }

            for link in &header.links {
                link.verify(project, "header", errors);
            }
        }
    }

    fn verify_footer(&self, project: &Project, errors: &mut Vec<Error>) {
        if let Some(footer) = self.footer() {
            for link in &footer.links {
                link.verify(project, "footer", errors);
            }
        }
    }

    fn verify_v2_theme(&self, errors: &mut Vec<Error>) {
        if let Some(false) = self
            .theme()
            .map(|t| t.colors.original_accent.clone())
            .map(|s| is_hex_color(&s))
        {
            errors.push(Error {
                code: Error::INVALID_DOCTAVE_YAML,
                message: String::from("Invalid accent color in theme"),
                description: format!("Expected a HEX color code, or a valid CSS color name."),
                file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                position: None,
            });
        }
    }

    fn verify_openapi_specs(&self, project: &Project, errors: &mut Vec<Error>) {
        for o in self.open_api() {
            if o.uri_prefix == "/" {
                errors.push(Error {
                    code: Error::INVALID_DOCTAVE_YAML,
                    message: String::from("OpenAPI URI prefix should contain a path."),
                    description: format!(
                        "Define a uri_prefix for the OpenAPI spec \"{}\" in docapella.yaml. For example, uri_prefix: /api.",
                        &o.spec_file.display()
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
            position: None,
                });
            }

            if !project.input_paths.contains(&o.spec_file) {
                errors.push(Error {
                    code: Error::INVALID_DOCTAVE_YAML,
                    message: String::from("Could not find OpenAPI spec."),
                    description: format!(
                        "OpenAPI spec at \"{}\" not found. Is it in the correct location?",
                        &o.spec_file.display()
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                });
            }
        }
    }

    fn verify_logo(&self, project: &Project, errors: &mut Vec<Error>) {
        if let Some(logo) = &self.logo() {
            logo.verify(project, errors);
        }
    }

    fn verify_favicon(&self, project: &Project, errors: &mut Vec<Error>) {
        if let Some(favicon) = self.favicon() {
            if !project.assets.iter().any(|asset| asset == &favicon.src) {
                errors.push(Error {
                    code: Error::INVALID_DOCTAVE_YAML,
                    message: format!(
                        "Could not find favicon at \"{}\".",
                        favicon.src.display()
                    ),
                    description: format!(
                        "Found following possible favicons: [{}].\nMake sure the file name is correct and located under the \"_assets\" directory.",
                        project.assets
                            .iter()
                            .filter(|a| {
                                let ext = a.path.extension().and_then(|s| s.to_str());
                                ext == Some("png")  || ext == Some("svg")  || ext == Some("ico")
                            })
                            .map(|s| format!("\"{}\"", s.path.display()))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: Some(PathBuf::from(crate::SETTINGS_FILE_NAME)),
            position: None,
                });
            }
        }
    }

    fn verify_styles(&self, project: &Project, errors: &mut Vec<Error>) {
        for path in self.styles() {
            if !project.assets.iter().any(|asset| asset == path) {
                errors.push(Error {
                    code: Error::INVALID_DOCTAVE_YAML,
                    message: format!(
                        "Could not find style sheet file at \"{}\".",
                        path.display()
                    ),
                    description: format!(
                        "Found [{}].\nMake sure the file name is correct and located under the \"_assets\" directory.",
                        project.assets
                            .iter()
                            .filter(|a| a.path.extension().and_then(|s| s.to_str()) == Some("css"))
                            .map(|s| format!("\"{}\"", s.path.display()))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: Some(PathBuf::from(crate::SETTINGS_FILE_NAME)),
            position: None,
                });
            }
        }
    }

    fn verify_redirects(&self, project: &Project, errors: &mut Vec<Error>) {
        let pages = project.pages();

        for (from, to) in self.redirects().iter().map(|r| r.as_tuple()) {
            let from_without_anchor = from.split('#').collect::<Vec<_>>()[0];
            let to_without_anchor = to.split('#').collect::<Vec<_>>()[0];

            if from_without_anchor.contains(':') {
                errors.push(Error {
                    code: Error::INVALID_REDIRECT,
                    message: String::from("Invalid redirect source"),
                    description: format!(
                        r#"Redirect source "{}" can't include path parameters."#,
                        from_without_anchor
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                });
            }

            if from_without_anchor.contains('*') && !WILDCARD_REGEX.is_match(from_without_anchor) {
                errors.push(Error {
                      code: Error::INVALID_REDIRECT,
                      message: String::from("Invalid redirect source"),
                      description: format!(r#"Redirect source "{}" with a wildcard should end with `.../*` or `.../**`."#, from),
                      file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
            position: None,
                  });
            }

            if pages.iter().any(|p| p.uri_path() == from_without_anchor) {
                errors.push(Error {
                    code: Error::INVALID_REDIRECT,
                    message: String::from("Redirect overlaps with existing page"),
                    description: format!(r#"Redirect source "{}" already exists as a page. Delete or rename the page, or change the redirect source."#, from_without_anchor),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
            position: None,
                });
            }

            if !from_without_anchor.starts_with('/') {
                errors.push(Error {
                    code: Error::INVALID_REDIRECT,
                    message: String::from("Invalid redirect detected"),
                    description: format!(
                        r#"Redirect source "{}" must start with a forward slash."#,
                        from_without_anchor
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                });
            }

            if from_without_anchor == to_without_anchor {
                errors.push(Error {
                    code: Error::INVALID_REDIRECT,
                    message: String::from("Invalid redirect detected"),
                    description: format!(
                        r#"Redirect source "{}" and destination "{}" must be different."#,
                        from_without_anchor, to_without_anchor
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                });
            }

            if from_without_anchor.ends_with(".md") {
                errors.push(Error {
                    code: Error::INVALID_REDIRECT,
                    message: String::from("Invalid redirect detected"),
                    description: format!(
                        r#"Redirect source "{}" must not end with .md."#,
                        from_without_anchor
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                });
            }

            let is_external = Url::parse(to_without_anchor).is_ok();

            if !is_external {
                if to_without_anchor.contains(':') {
                    if !PATH_REGEX.is_match(to_without_anchor) {
                        errors.push(Error {
                        code: Error::INVALID_REDIRECT,
                        message: String::from("Invalid redirect destination"),
                        description: format!(
                            r#"Redirect destination "{}" path parameters can only contain alphanumerics and underscores."#,
                            to
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
            position: None,
                    });
                    }

                    if !from_without_anchor.contains('*') {
                        errors.push(Error {
                          code: Error::INVALID_REDIRECT,
                          message: String::from("Invalid redirect source"),
                          description: format!(r#"Redirect source "{}" should include a wildcard when `to` has path parameters."#, from),
                          file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
            position: None,
                      });
                    }
                }

                if !to.contains(':') && !pages.iter().any(|p| p.uri_path() == to_without_anchor) {
                    errors.push(Error {
                        code: Error::INVALID_REDIRECT,
                        message: String::from("Broken redirect detected"),
                        description: format!(
                            r#"Redirect destination "{}" does not exist."#,
                            to_without_anchor
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }

                if !to_without_anchor.starts_with('/') {
                    errors.push(Error {
                        code: Error::INVALID_REDIRECT,
                        message: String::from("Invalid redirect detected"),
                        description: format!(
                            r#"Redirect destination "{}" must start with a forward slash, or be an external URL."#,
                            to_without_anchor
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }

                if to_without_anchor.ends_with(".md") {
                    errors.push(Error {
                        code: Error::INVALID_REDIRECT,
                        message: String::from("Invalid redirect detected"),
                        description: format!(
                            r#"Redirect destination "{}" must not end with .md."#,
                            to_without_anchor
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Radius {
    None,
    Small,
    #[default]
    Medium,
    Large,
    Full,
}

impl Display for Radius {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use Radius::*;
        match self {
            None => write!(f, "none"),
            Small => write!(f, "small"),
            Medium => write!(f, "medium"),
            Large => write!(f, "large"),
            Full => write!(f, "full"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Settings {
    pub title: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub header: Option<HeaderSettings>,
    #[serde(default = "Theme::default")]
    pub theme: Theme,
    #[serde(default)]
    pub open_api: Vec<OpenApi>,
    #[serde(default)]
    pub styles: Vec<PathBuf>,
    #[serde(default)]
    pub redirects: Vec<Redirect>,
    #[serde(default, rename(deserialize = "tabs"))]
    pub tab_descriptions: Vec<TabDescription>,
    #[serde(default)]
    pub footer: Footer,
    #[serde(default)]
    pub vale: Option<ValeSettings>,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            title: String::from("unknown project"),
            header: None,
            theme: Theme::default(),
            open_api: Vec::new(),
            styles: Vec::new(),
            redirects: Vec::new(),
            tab_descriptions: Vec::new(),
            footer: Footer::default(),
            vale: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValeSettings {
    pub version: Option<String>,
    pub config_file_path: Option<String>,
}

pub type FooterLink = HeaderLink;

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Footer {
    #[serde(default)]
    pub links: Vec<FooterLink>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    linkedin: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    twitter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    github: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    facebook: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    discord: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
#[serde(tag = "name")]
pub struct Theme {
    #[serde(default)]
    pub color_mode: ColorMode,
    #[serde(default)]
    pub colors: ColorsV2,
    pub logo: Option<Logo>,
    #[serde(default)]
    pub radius: Radius,
    pub favicon: Option<Favicon>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ColorsV2Description {
    #[serde(default)]
    accent: String,
    #[serde(default)]
    grayscale: Grayscale,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(from = "ColorsV2Description")]
pub struct ColorsV2 {
    #[serde(default)]
    accent: String,
    #[serde(skip)]
    original_accent: String,
    #[serde(default)]
    grayscale: Grayscale,
}

impl From<ColorsV2Description> for ColorsV2 {
    fn from(value: ColorsV2Description) -> Self {
        let accent = if !is_hex_color(&value.accent) {
            ColorsV2::default().accent.clone()
        } else {
            value.accent.clone()
        };

        ColorsV2 {
            accent,
            original_accent: value.accent.clone(),
            grayscale: value.grayscale,
        }
    }
}

impl Default for ColorsV2 {
    fn default() -> Self {
        ColorsV2 {
            accent: "#5B5BD6".to_string(), // Radix Iris 9
            original_accent: "#5B5BD6".to_string(),
            grayscale: Grayscale::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub enum Grayscale {
    #[default]
    #[serde(rename = "gray")]
    Gray,
    #[serde(rename = "mauve")]
    Mauve,
    #[serde(rename = "slate")]
    Slate,
    #[serde(rename = "sage")]
    Sage,
    #[serde(rename = "olive")]
    Olive,
    #[serde(rename = "sand")]
    Sand,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ColorMode {
    #[serde(rename = "auto")]
    #[default]
    Auto,
    #[serde(rename = "dark")]
    Dark,
    #[serde(rename = "light")]
    Light,
}

#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HeaderSettings {
    #[serde(default)]
    pub links: Vec<HeaderLink>,
    pub cta: Option<HeaderCta>,
}

/// A intermediary struct use to parse colors.
///
/// The purpose is to be able to compute default values for the
/// section specific colors.
///
/// If you only specify the `border` color, it should get propagated
/// to the border colors of each section, while still letting specific
/// sections override this default.
///
/// See `TryFrom<ColorsDescription> for Color` for the logic around this.
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ColorsDescription {
    #[serde(default = "Colors::default_main")]
    pub main: String,
    #[serde(default = "Colors::default_main_contrast")]
    pub main_contrast: String,
    #[serde(default = "Colors::default_text_base")]
    pub text_base: String,
    #[serde(default = "Colors::default_text_soft")]
    pub text_soft: String,
    #[serde(default = "Colors::default_text_strong")]
    pub text_strong: String,
    #[serde(default = "Colors::default_bg")]
    pub bg: String,
    #[serde(default = "Colors::default_bg_secondary")]
    pub bg_secondary: String,
    #[serde(default = "Colors::default_border")]
    pub border: String,
    #[serde(default = "Colors::default_main_dark")]
    pub main_dark: String,
    #[serde(default = "Colors::default_main_contrast_dark")]
    pub main_contrast_dark: String,
    #[serde(default = "Colors::default_text_base_dark")]
    pub text_base_dark: String,
    #[serde(default = "Colors::default_text_soft_dark")]
    pub text_soft_dark: String,
    #[serde(default = "Colors::default_text_strong_dark")]
    pub text_strong_dark: String,
    #[serde(default = "Colors::default_bg_dark")]
    pub bg_dark: String,
    #[serde(default = "Colors::default_bg_secondary_dark")]
    pub bg_secondary_dark: String,
    #[serde(default = "Colors::default_border_dark")]
    pub border_dark: String,
    pub content_main: Option<String>,
    pub content_main_dark: Option<String>,
    pub content_main_contrast: Option<String>,
    pub content_main_contrast_dark: Option<String>,
    pub content_bg: Option<String>,
    pub content_bg_dark: Option<String>,
    pub content_bg_secondary: Option<String>,
    pub content_bg_secondary_dark: Option<String>,
    pub content_text_base: Option<String>,
    pub content_text_base_dark: Option<String>,
    pub content_text_strong: Option<String>,
    pub content_text_strong_dark: Option<String>,
    pub content_text_soft: Option<String>,
    pub content_text_soft_dark: Option<String>,
    pub content_border: Option<String>,
    pub content_border_dark: Option<String>,
    pub nav_main: Option<String>,
    pub nav_main_dark: Option<String>,
    pub nav_main_contrast: Option<String>,
    pub nav_main_contrast_dark: Option<String>,
    pub nav_bg: Option<String>,
    pub nav_bg_dark: Option<String>,
    pub nav_bg_secondary: Option<String>,
    pub nav_bg_secondary_dark: Option<String>,
    pub nav_text_base: Option<String>,
    pub nav_text_base_dark: Option<String>,
    pub nav_text_strong: Option<String>,
    pub nav_text_strong_dark: Option<String>,
    pub nav_text_soft: Option<String>,
    pub nav_text_soft_dark: Option<String>,
    pub nav_border: Option<String>,
    pub nav_border_dark: Option<String>,
    pub header_main: Option<String>,
    pub header_main_dark: Option<String>,
    pub header_main_contrast: Option<String>,
    pub header_main_contrast_dark: Option<String>,
    pub header_bg: Option<String>,
    pub header_bg_secondary: Option<String>,
    pub header_bg_dark: Option<String>,
    pub header_bg_secondary_dark: Option<String>,
    pub header_text_base: Option<String>,
    pub header_text_base_dark: Option<String>,
    pub header_text_strong: Option<String>,
    pub header_text_strong_dark: Option<String>,
    pub header_text_soft: Option<String>,
    pub header_text_soft_dark: Option<String>,
    pub header_border: Option<String>,
    pub header_border_dark: Option<String>,
}

impl TryFrom<ColorsDescription> for Colors {
    type Error = std::num::ParseIntError;

    fn try_from(other: ColorsDescription) -> std::result::Result<Self, Self::Error> {
        Ok(Colors {
            main: other.main.clone(),
            main_dark: other.main_dark.clone(),
            main_contrast: other.main_contrast.clone(),
            main_contrast_dark: other.main_contrast_dark.clone(),
            content_main: other.content_main.unwrap_or_else(|| other.main.clone()),
            content_main_dark: other
                .content_main_dark
                .unwrap_or_else(|| other.main_dark.clone()),
            content_main_contrast: other
                .content_main_contrast
                .unwrap_or_else(|| other.main_contrast.clone()),
            content_main_contrast_dark: other
                .content_main_contrast_dark
                .unwrap_or_else(|| other.main_contrast_dark.clone()),
            content_bg: other.content_bg.unwrap_or_else(|| other.bg.clone()),
            content_bg_secondary: other
                .content_bg_secondary
                .unwrap_or_else(|| other.bg_secondary.clone()),
            content_bg_dark: other
                .content_bg_dark
                .unwrap_or_else(|| other.bg_dark.clone()),
            content_bg_secondary_dark: other
                .content_bg_secondary_dark
                .unwrap_or_else(|| other.bg_secondary_dark.clone()),
            content_text_base: other
                .content_text_base
                .unwrap_or_else(|| other.text_base.clone()),
            content_text_base_dark: other
                .content_text_base_dark
                .unwrap_or_else(|| other.text_base_dark.clone()),
            content_text_strong: other
                .content_text_strong
                .unwrap_or_else(|| other.text_strong.clone()),
            content_text_strong_dark: other
                .content_text_strong_dark
                .unwrap_or_else(|| other.text_strong_dark.clone()),
            content_text_soft: other
                .content_text_soft
                .unwrap_or_else(|| other.text_soft.clone()),
            content_text_soft_dark: other
                .content_text_soft_dark
                .unwrap_or_else(|| other.text_soft_dark.clone()),
            content_border: other.content_border.unwrap_or_else(|| other.border.clone()),
            content_border_dark: other
                .content_border_dark
                .unwrap_or_else(|| other.border_dark.clone()),
            nav_main: other.nav_main.unwrap_or_else(|| other.main.clone()),
            nav_main_dark: other
                .nav_main_dark
                .unwrap_or_else(|| other.main_dark.clone()),
            nav_main_contrast: other
                .nav_main_contrast
                .unwrap_or_else(|| other.main_contrast.clone()),
            nav_main_contrast_dark: other
                .nav_main_contrast_dark
                .unwrap_or_else(|| other.main_contrast_dark.clone()),
            nav_bg: other.nav_bg.unwrap_or_else(|| other.bg.clone()),
            nav_bg_dark: other.nav_bg_dark.unwrap_or_else(|| other.bg_dark.clone()),
            nav_bg_secondary: other
                .nav_bg_secondary
                .unwrap_or_else(|| other.bg_secondary.clone()),
            nav_bg_secondary_dark: other
                .nav_bg_secondary_dark
                .unwrap_or_else(|| other.bg_secondary_dark.clone()),
            nav_text_base: other
                .nav_text_base
                .unwrap_or_else(|| other.text_base.clone()),
            nav_text_base_dark: other
                .nav_text_base_dark
                .unwrap_or_else(|| other.text_base_dark.clone()),
            nav_text_strong: other
                .nav_text_strong
                .unwrap_or_else(|| other.text_strong.clone()),
            nav_text_strong_dark: other
                .nav_text_strong_dark
                .unwrap_or_else(|| other.text_strong_dark.clone()),
            nav_text_soft: other
                .nav_text_soft
                .unwrap_or_else(|| other.text_soft.clone()),
            nav_text_soft_dark: other
                .nav_text_soft_dark
                .unwrap_or_else(|| other.text_soft_dark.clone()),
            nav_border: other.nav_border.unwrap_or_else(|| other.border.clone()),
            nav_border_dark: other
                .nav_border_dark
                .unwrap_or_else(|| other.border_dark.clone()),
            header_main: other.header_main.unwrap_or_else(|| other.main.clone()),
            header_main_dark: other
                .header_main_dark
                .unwrap_or_else(|| other.main_dark.clone()),
            header_main_contrast: other
                .header_main_contrast
                .unwrap_or_else(|| other.main_contrast.clone()),
            header_main_contrast_dark: other
                .header_main_contrast_dark
                .unwrap_or_else(|| other.main_contrast_dark.clone()),
            header_bg: other.header_bg.unwrap_or_else(|| other.bg.clone()),
            header_bg_secondary: other
                .header_bg_secondary
                .unwrap_or_else(|| other.bg_secondary.clone()),
            header_bg_dark: other
                .header_bg_dark
                .unwrap_or_else(|| other.bg_dark.clone()),
            header_bg_secondary_dark: other
                .header_bg_secondary_dark
                .unwrap_or_else(|| other.bg_secondary_dark.clone()),
            header_text_base: other
                .header_text_base
                .unwrap_or_else(|| other.text_base.clone()),
            header_text_base_dark: other
                .header_text_base_dark
                .unwrap_or_else(|| other.text_base_dark.clone()),
            header_text_strong: other
                .header_text_strong
                .unwrap_or_else(|| other.text_strong.clone()),
            header_text_strong_dark: other
                .header_text_strong_dark
                .unwrap_or_else(|| other.text_strong_dark.clone()),
            header_text_soft: other
                .header_text_soft
                .unwrap_or_else(|| other.text_soft.clone()),
            header_text_soft_dark: other
                .header_text_soft_dark
                .unwrap_or_else(|| other.text_soft_dark.clone()),
            header_border: other.header_border.unwrap_or_else(|| other.border.clone()),
            header_border_dark: other
                .header_border_dark
                .unwrap_or_else(|| other.border_dark.clone()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(try_from = "ColorsDescription")]
#[serde(deny_unknown_fields)]
pub struct Colors {
    pub main: String,
    pub main_dark: String,
    pub main_contrast: String,
    pub main_contrast_dark: String,
    pub content_main: String,
    pub content_main_dark: String,
    pub content_main_contrast: String,
    pub content_main_contrast_dark: String,
    pub content_bg: String,
    pub content_bg_secondary: String,
    pub content_bg_dark: String,
    pub content_bg_secondary_dark: String,
    pub content_text_base: String,
    pub content_text_base_dark: String,
    pub content_text_strong: String,
    pub content_text_strong_dark: String,
    pub content_text_soft: String,
    pub content_text_soft_dark: String,
    pub content_border: String,
    pub content_border_dark: String,
    pub nav_main: String,
    pub nav_main_dark: String,
    pub nav_main_contrast: String,
    pub nav_main_contrast_dark: String,
    pub nav_bg: String,
    pub nav_bg_dark: String,
    pub nav_bg_secondary: String,
    pub nav_bg_secondary_dark: String,
    pub nav_text_base: String,
    pub nav_text_base_dark: String,
    pub nav_text_strong: String,
    pub nav_text_strong_dark: String,
    pub nav_text_soft: String,
    pub nav_text_soft_dark: String,
    pub nav_border: String,
    pub nav_border_dark: String,
    pub header_main: String,
    pub header_main_dark: String,
    pub header_main_contrast: String,
    pub header_main_contrast_dark: String,
    pub header_bg: String,
    pub header_bg_secondary: String,
    pub header_bg_dark: String,
    pub header_bg_secondary_dark: String,
    pub header_text_base: String,
    pub header_text_base_dark: String,
    pub header_text_strong: String,
    pub header_text_strong_dark: String,
    pub header_text_soft: String,
    pub header_text_soft_dark: String,
    pub header_border: String,
    pub header_border_dark: String,
}

impl Default for Colors {
    fn default() -> Self {
        Colors {
            main: Self::default_main(),
            main_dark: Self::default_main_dark(),
            main_contrast: Self::default_main_contrast(),
            main_contrast_dark: Self::default_main_contrast_dark(),
            content_main: Self::default_main(),
            content_main_dark: Self::default_main_dark(),
            content_main_contrast: Self::default_main_contrast(),
            content_main_contrast_dark: Self::default_main_contrast_dark(),
            content_bg: Self::default_bg(),
            content_bg_secondary: Self::default_bg_secondary(),
            content_bg_dark: Self::default_bg_dark(),
            content_bg_secondary_dark: Self::default_bg_secondary_dark(),
            content_text_base: Self::default_text_base(),
            content_text_base_dark: Self::default_text_base_dark(),
            content_text_strong: Self::default_text_strong(),
            content_text_strong_dark: Self::default_text_strong_dark(),
            content_text_soft: Self::default_text_soft(),
            content_text_soft_dark: Self::default_text_soft_dark(),
            content_border: Self::default_border(),
            content_border_dark: Self::default_border_dark(),
            nav_main: Self::default_main(),
            nav_main_dark: Self::default_main_dark(),
            nav_main_contrast: Self::default_main_contrast(),
            nav_main_contrast_dark: Self::default_main_contrast_dark(),
            nav_bg: Self::default_bg(),
            nav_bg_secondary: Self::default_bg_secondary(),
            nav_bg_dark: Self::default_bg_dark(),
            nav_bg_secondary_dark: Self::default_bg_secondary_dark(),
            nav_text_base: Self::default_text_base(),
            nav_text_base_dark: Self::default_text_base_dark(),
            nav_text_strong: Self::default_text_strong(),
            nav_text_strong_dark: Self::default_text_strong_dark(),
            nav_text_soft: Self::default_text_soft(),
            nav_text_soft_dark: Self::default_text_soft_dark(),
            nav_border: Self::default_border(),
            nav_border_dark: Self::default_border_dark(),
            header_main: Self::default_main(),
            header_main_dark: Self::default_main_dark(),
            header_main_contrast: Self::default_main_contrast(),
            header_main_contrast_dark: Self::default_main_contrast_dark(),
            header_bg: Self::default_bg(),
            header_bg_secondary: Self::default_bg_secondary(),
            header_bg_dark: Self::default_bg_dark(),
            header_bg_secondary_dark: Self::default_bg_secondary_dark(),
            header_text_base: Self::default_text_base(),
            header_text_base_dark: Self::default_text_base_dark(),
            header_text_strong: Self::default_text_strong(),
            header_text_strong_dark: Self::default_text_strong_dark(),
            header_text_soft: Self::default_text_soft(),
            header_text_soft_dark: Self::default_text_soft_dark(),
            header_border: Self::default_border(),
            header_border_dark: Self::default_border_dark(),
        }
    }
}

impl Colors {
    pub fn serialize(&self) -> String {
        serde_yaml::to_string(self)
            .unwrap()
            .replace('\n', "\n  ")
            .trim_end_matches("\n  ")
            .to_string()
    }

    // cyan-700
    fn default_main() -> String {
        "#7B8FFE".to_owned()
    }

    // white
    fn default_main_contrast() -> String {
        "#FFFFFF".to_owned()
    }

    // slate-600
    fn default_text_base() -> String {
        "#475569".to_owned()
    }

    // slate-400
    fn default_text_soft() -> String {
        "#94a3b8".to_owned()
    }

    // slate-800
    fn default_text_strong() -> String {
        "#1e293b".to_owned()
    }

    // white
    fn default_bg() -> String {
        "#FFF".to_owned()
    }

    // slate-50
    fn default_bg_secondary() -> String {
        "#f8fafc".to_owned()
    }

    // slate-200
    fn default_border() -> String {
        "#e2e8f0".to_owned()
    }

    // cyan-700
    fn default_main_dark() -> String {
        "#7B8FFE".to_owned()
    }

    // white
    fn default_main_contrast_dark() -> String {
        "#FFFFFF".to_owned()
    }

    // slate-900
    fn default_bg_dark() -> String {
        "#0F172A".to_owned()
    }

    // slate-800
    fn default_bg_secondary_dark() -> String {
        "#1E293B".to_owned()
    }

    // slate-300
    fn default_text_base_dark() -> String {
        "#cbd5e1".to_owned()
    }

    // slate-50
    fn default_text_strong_dark() -> String {
        "#f8fafc".to_owned()
    }

    // slate-500
    fn default_text_soft_dark() -> String {
        "#64748b".to_owned()
    }

    // slate-700
    fn default_border_dark() -> String {
        "#334155".to_owned()
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Logo {
    pub src: PathBuf,
    pub src_dark: Option<PathBuf>,
}

impl Logo {
    pub(crate) fn verify(&self, project: &Project, errors: &mut Vec<Error>) {
        if !project.assets.iter().any(|asset| asset == &self.src) {
            errors.push(Error {
                    code: Error::INVALID_DOCTAVE_YAML,
                    message: format!(
                        "Could not find logo at \"{}\".",
                        self.src.display()
                    ),
                    description: format!(
                        "Found following images: [{}].\nMake sure the file name is correct and located under the \"_assets\" directory.",
                        project.assets
                            .iter()
                            .filter(|a| {
                                let ext = a.path.extension().and_then(|s| s.to_str());
                                ext == Some("jpg") || ext == Some("png")  || ext == Some("jpeg") || ext == Some("svg")
                            })
                            .map(|s| format!("\"{}\"", s.path.display()))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: Some(PathBuf::from(crate::SETTINGS_FILE_NAME)),
            position: None,
                });
        }

        if let Some(dark_src) = &self.src_dark {
            if !project.assets.iter().any(|asset| asset == dark_src) {
                errors.push(Error {
                    code: Error::INVALID_DOCTAVE_YAML,
                    message: format!(
                        "Could not find dark mode logo at \"{}\".",
                        dark_src.display()
                    ),
                    description: format!(
                        "Found following images: [{}].\nMake sure the file name is correct and located under the \"_assets\" directory.",
                        project.assets
                            .iter()
                            .filter(|a| {
                                let ext = a.path.extension().and_then(|s| s.to_str());
                                ext == Some("jpg") || ext == Some("png")  || ext == Some("jpeg") || ext == Some("svg")
                            })
                            .map(|s| format!("\"{}\"", s.path.display()))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: Some(PathBuf::from(crate::SETTINGS_FILE_NAME)),
            position: None,
                });
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Redirect {
    pub from: String,
    pub to: String,
}

impl Redirect {
    pub fn as_tuple(&self) -> (String, String) {
        (self.from.clone(), self.to.clone())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Favicon {
    pub src: PathBuf,
}

type HeaderCta = HeaderLink;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum HeaderLink {
    Internal(InternalLink),
    External(ExternalLink),
}

impl HeaderLink {
    pub fn verify(&self, project: &Project, verify_as: &str, errors: &mut Vec<Error>) {
        match self {
            HeaderLink::Internal(link) => {
                if link.download {
                    let path = PathBuf::from(&link.href);
                    if project.get_asset_by_fs_path(&path).is_none() {
                        let thing = match verify_as {
                            "footer" => "Footer",
                            _ => "Header",
                        };
                        let error = Error {
                            code: Error::BROKEN_INTERNAL_LINK,
                            message: String::from("Broken asset link detected"),
                            description: format!(
                                r#"{thing} link "{}" points to an unknown file."#,
                                link.href
                            ),
                            file: Some(SETTINGS_FILE_NAME.into()),
                            position: None,
                        };

                        errors.push(error);
                    }
                } else {
                    let path = PathBuf::from(&link.href);
                    let uri = crate::fs_to_uri_path(&path);

                    if project.get_page_by_uri_path(&uri).is_none()
                        && !project.redirects().iter().any(|r| r.0 == uri)
                    {
                        let thing = match verify_as {
                            "footer" => "Footer",
                            _ => "Header",
                        };
                        let error = Error {
                            code: Error::BROKEN_INTERNAL_LINK,
                            message: String::from("Broken link detected"),
                            description: format!(
                                r#"{thing} link "{}" points to an unknown file."#,
                                link.href
                            ),
                            file: Some(SETTINGS_FILE_NAME.into()),
                            position: None,
                        };

                        errors.push(error);
                    }
                }
            }
            HeaderLink::External(link) => {
                if !is_external_link(&link.external) {
                    let thing = match verify_as {
                        "footer" => "footer",
                        _ => "header",
                    };
                    errors.push(Error {
                        code: Error::INVALID_DOCTAVE_YAML,
                        message: format!("Invalid {thing} link in found in docapella.yaml"),
                        description: format!(
                            "Found \"{}\", which is not an external link.",
                            &link.external
                        ),
                        file: Some(SETTINGS_FILE_NAME.into()),
                        position: None,
                    })
                }
            }
        }
    }

    pub fn unwrap_as_external(&self) -> &ExternalLink {
        match self {
            HeaderLink::External(link) => link,
            _ => panic!("not an external link"),
        }
    }

    pub fn unwrap_as_internal(&self) -> &InternalLink {
        match self {
            HeaderLink::Internal(link) => link,
            _ => panic!("not an internal link"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct InternalLink {
    #[serde(alias = "text")]
    pub label: String,
    pub href: String,
    #[serde(default)]
    pub download: bool,
    #[serde(default)]
    pub download_as: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ExternalLink {
    #[serde(alias = "text")]
    pub label: String,
    pub external: String,
}

fn normalize_uri_path<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer)
        .map(|s: &str| format!("/{}", s.trim_start_matches('/').trim_end_matches('/')))
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OpenApi {
    pub spec_file: PathBuf,
    #[serde(deserialize_with = "normalize_uri_path")]
    pub uri_prefix: String,
    #[serde(default)]
    pub experimental: bool,
}

/// Check if a string is a valid hex color.
///
/// Expects the first character to be `#`, followed by 6 or 8 hex digits.
fn is_hex_color(s: &str) -> bool {
    s.starts_with('#') && s.chars().skip(1).all(|c| c.is_ascii_hexdigit())
}

#[cfg(test)]
mod test {

    use super::*;
    use std::path::Path;

    #[test]
    fn rewrite_logo_and_favicon_links() {
        let input = indoc! {r##"
        ---
        title: Acme Inc

        theme:
          logo:
            src: _assets/logo.png
            src_dark: _assets/logo-dark.png

          favicon:
            src: _assets/favicon.svg
        "##};

        let mut settings = Settings::parse(input).unwrap();

        settings.rewrite_links(
            &RenderOptions {
                link_rewrites: [
                    // NOTE: Jaleo always adds the "/" prefix
                    ("/_assets/logo.png".to_string(), "new_logo.png".to_string()),
                    (
                        "/_assets/logo-dark.png".to_string(),
                        "new_logo_dark.png".to_string(),
                    ),
                    (
                        "/_assets/favicon.svg".to_string(),
                        "new_favicon.svg".to_string(),
                    ),
                ]
                .into(),
                ..Default::default()
            },
            &[],
        );

        assert_eq!(&settings.logo().unwrap().src, &Path::new("new_logo.png"));
        assert_eq!(
            settings.logo().unwrap().src_dark.as_deref(),
            Some(Path::new("new_logo_dark.png"))
        );
        assert_eq!(
            &settings.favicon().unwrap().src,
            &Path::new("new_favicon.svg")
        );
    }

    #[test]
    fn rewrite_v2_logo_and_favicon_links() {
        let input = indoc! {r##"
        ---
        title: Acme Inc

        theme:
          logo:
            src: _assets/logo.png
            src_dark: _assets/logo-dark.png

          favicon:
            src: _assets/favicon.svg
        "##};

        let mut settings = Settings::parse(input).unwrap();

        settings.rewrite_links(
            &RenderOptions {
                link_rewrites: [
                    // NOTE: Jaleo always adds the "/" prefix
                    ("/_assets/logo.png".to_string(), "new_logo.png".to_string()),
                    (
                        "/_assets/logo-dark.png".to_string(),
                        "new_logo_dark.png".to_string(),
                    ),
                    (
                        "/_assets/favicon.svg".to_string(),
                        "new_favicon.svg".to_string(),
                    ),
                ]
                .into(),
                ..Default::default()
            },
            &[],
        );

        assert_eq!(&settings.logo().unwrap().src, &Path::new("new_logo.png"));
        assert_eq!(
            settings.logo().unwrap().src_dark.as_deref(),
            Some(Path::new("new_logo_dark.png"))
        );
        assert_eq!(
            &settings.favicon().unwrap().src,
            &Path::new("new_favicon.svg")
        );
    }

    #[test]
    fn v2_has_a_default_color() {
        let input = indoc! {r##"
        ---
        title: Acme Inc
        "##};

        let settings = Settings::parse(input).unwrap();
        assert_eq!(settings.theme().unwrap().colors.accent, "#5B5BD6");
    }

    #[test]
    fn v2_has_a_default_grayscale() {
        let input = indoc! {r##"
        ---
        title: Acme Inc
        "##};

        let settings = Settings::parse(input).unwrap();
        assert_eq!(settings.theme().unwrap().colors.grayscale, Grayscale::Gray);
    }

    #[test]
    fn v2_doesnt_explode_with_invalid_color() {
        let input = indoc! {r##"
        ---
        title: Acme Inc

        theme:
          colors:
            accent: "lolnotacolor"
        "##};

        let settings = Settings::parse(input).unwrap();
        // Falls back to default
        assert_eq!(settings.theme().unwrap().colors.accent, "#5B5BD6");
    }

    #[test]
    fn rewrite_logo_and_favicon_links_that_are_prefixed_with_slashes() {
        let input = indoc! {r##"
        ---
        title: Acme Inc

        theme:
          logo:
            src: /_assets/logo.png
            src_dark: /_assets/logo-dark.png

          favicon:
            src: /_assets/favicon.svg
        "##};

        let mut settings = Settings::parse(input).unwrap();

        settings.rewrite_links(
            &RenderOptions {
                link_rewrites: [
                    // NOTE: Jaleo always adds the "/" prefix
                    ("/_assets/logo.png".to_string(), "new_logo.png".to_string()),
                    (
                        "/_assets/logo-dark.png".to_string(),
                        "new_logo_dark.png".to_string(),
                    ),
                    (
                        "/_assets/favicon.svg".to_string(),
                        "new_favicon.svg".to_string(),
                    ),
                ]
                .into(),
                ..Default::default()
            },
            &[],
        );

        assert_eq!(&settings.logo().unwrap().src, &Path::new("new_logo.png"));
        assert_eq!(
            settings.logo().unwrap().src_dark.as_deref(),
            Some(Path::new("new_logo_dark.png"))
        );
        assert_eq!(
            &settings.favicon().unwrap().src,
            &Path::new("new_favicon.svg")
        );
    }

    #[test]
    fn fails_on_unknown_colors() {
        let input = indoc! {"
        ---
        title: Acme Inc

        colors:
            wat: \"#CECECE\"
        "};

        assert!(
            serde_yaml::from_str::<Settings>(input).is_err(),
            "Failed to recognise unknown color"
        );
    }

    #[test]
    fn from_serde_all_attributes() {
        let input = indoc! {"
        ---
        title: Acme Inc

        theme:
          colors:
            main: \"#CECECE\"

        header:
          links:
            - text: Google
              external: https://www.google.com
            - text: Example
              external: https://www.example.com
          cta:
            text: Sign Up
            external: https://www.example.com


        open_api:
            - spec_file: /path/to/spec.json
              uri_prefix: /bobby
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(settings.title, String::from("Acme Inc"));

        // V1 color test removed - colors.main no longer exists in V2

        let header = settings.header.unwrap();

        assert_eq!(
            header.links[0].unwrap_as_external().label,
            String::from("Google")
        );
        assert_eq!(
            header.links[0].unwrap_as_external().external,
            String::from("https://www.google.com")
        );

        assert_eq!(
            header.links[1].unwrap_as_external().label,
            String::from("Example")
        );
        assert_eq!(
            header.links[1].unwrap_as_external().external,
            String::from("https://www.example.com")
        );

        assert_eq!(
            header.cta.unwrap().unwrap_as_external(),
            &ExternalLink {
                label: "Sign Up".to_string(),
                external: "https://www.example.com".to_string()
            }
        );

        assert_eq!(
            settings.open_api[0].spec_file,
            Path::new("/path/to/spec.json")
        );
        assert_eq!(settings.open_api[0].uri_prefix, "/bobby");
    }

    #[test]
    fn trims_trailing_slash_from_openapi_uri_prefix() {
        let input = indoc! {"
        ---
        title: Acme Inc

        open_api:
            - spec_file: /path/to/spec.json
              uri_prefix: /bobby/
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(settings.open_api[0].uri_prefix, "/bobby");
    }

    #[test]
    fn adds_leading_slash_to_openapi_uri_prefix() {
        let input = indoc! {"
        ---
        title: Acme Inc

        open_api:
            - spec_file: /path/to/spec.json
              uri_prefix: bobby
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(settings.open_api[0].uri_prefix, "/bobby");
    }

    #[test]
    fn from_serde_header_label_alias() {
        let input = indoc! {"
        ---
        title: Acme Inc

        header:
            links:
                - label: Google
                  external: https://www.google.com
                - label: Example
                  external: https://www.example.com
            cta:
                label: Sign Up
                external: https://www.example.com
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(settings.title, String::from("Acme Inc"));

        let header = settings.header.unwrap();

        assert_eq!(
            header.links[0].unwrap_as_external().label,
            String::from("Google")
        );
        assert_eq!(
            header.links[0].unwrap_as_external().external,
            String::from("https://www.google.com")
        );

        assert_eq!(
            header.links[1].unwrap_as_external().label,
            String::from("Example")
        );
        assert_eq!(
            header.links[1].unwrap_as_external().external,
            String::from("https://www.example.com")
        );

        assert_eq!(
            header.cta.unwrap().unwrap_as_external(),
            &ExternalLink {
                label: "Sign Up".to_string(),
                external: "https://www.example.com".to_string()
            }
        );
    }

    #[test]
    fn from_serde_header_internal_links() {
        let input = indoc! {"
        ---
        title: Acme Inc

        header:
            links:
                - label: Google
                  href: /bar
                - label: Example
                  href: /foo
            cta:
                label: Sign Up
                href: /baz
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        let header = settings.header.unwrap();

        assert_eq!(
            header.links[0].unwrap_as_internal().label,
            String::from("Google")
        );
        assert_eq!(
            header.links[0].unwrap_as_internal().href,
            String::from("/bar")
        );

        assert_eq!(
            header.links[1].unwrap_as_internal().label,
            String::from("Example")
        );
        assert_eq!(
            header.links[1].unwrap_as_internal().href,
            String::from("/foo")
        );

        assert_eq!(
            header.cta.unwrap().unwrap_as_internal(),
            &InternalLink {
                label: "Sign Up".to_string(),
                href: "/baz".into(),
                download: false,
                download_as: "".into()
            }
        );
    }

    #[test]
    fn from_serde_header_download_links() {
        let input = indoc! {"
        ---
        title: Acme Inc

        header:
            links:
                - label: Google
                  href: /bar
                - label: Example
                  href: /foo
                  download: true
            cta:
                label: Sign Up
                href: /baz
                download: true
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        let header = settings.header.unwrap();

        assert_eq!(
            header.links[0].unwrap_as_internal().label,
            String::from("Google")
        );
        assert_eq!(
            header.links[0].unwrap_as_internal().href,
            String::from("/bar")
        );

        assert_eq!(
            header.links[1].unwrap_as_internal().label,
            String::from("Example")
        );
        assert_eq!(
            header.links[1].unwrap_as_internal().href,
            String::from("/foo")
        );
        assert!(header.links[1].unwrap_as_internal().download);

        assert_eq!(
            header.cta.unwrap().unwrap_as_internal(),
            &InternalLink {
                label: "Sign Up".to_string(),
                href: "/baz".into(),
                download: true,
                download_as: "".into()
            }
        );
    }

    #[test]
    fn styles() {
        let input = indoc! {"
        ---
        title: Acme Inc

        styles:
          - _assets/styles/style.css
        "};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(
            settings.styles[0],
            PathBuf::from("_assets/styles/style.css")
        );
    }

    #[test]
    fn color_mode_settings_get_set() {
        let input = indoc! {r##"
        ---
        title: Acme Inc

        theme:
          color_mode: "dark"
        "##};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(settings.theme.color_mode, ColorMode::Dark);
    }

    #[test]
    fn color_mode_settings_get_set_default() {
        let input = indoc! {r##"
        ---
        title: Acme Inc
        "##};

        let settings: Settings = serde_yaml::from_str(input).unwrap();

        assert_eq!(settings.theme.color_mode, ColorMode::Auto);
    }

    mod v2 {
        use super::*;

        #[test]
        fn basic() {
            let input = indoc! {r##"
        ---
        title: Acme Inc

        theme:
          logo:
            src: _assets/logo.svg
            src_dark: _assets/logo-dark.svg
          radius: large
          favicon:
            src: _assets/favicon.svg

        header:
          links:
            - label: Example
              external: https://www.example.com
          cta:
            text: Sign Up
            external: https://www.example.com

        open_api:
            - spec_file: /path/to/spec.json
              uri_prefix: /bobby

        styles:
            - _assets/style.css

        redirects:
            - from: /foo
              to: /bar

        footer:
          links:
            - label: Terms of Service
              external: http://www.example.com
            - label: Privacy Policy
              external: http://www.example.com
          linkedin: Doctave
          twitter: Doctave
          github: Doctave
        "##};

            let settings = Settings::parse(input).unwrap();

            assert!(true);
            assert_eq!(settings.title(), "Acme Inc");
            assert_eq!(
                settings.header(),
                Some(&HeaderSettings {
                    links: vec![HeaderLink::External(ExternalLink {
                        label: "Example".to_string(),
                        external: "https://www.example.com".to_string(),
                    })],
                    cta: Some(HeaderCta::External(ExternalLink {
                        label: "Sign Up".to_string(),
                        external: "https://www.example.com".to_string()
                    }))
                })
            );
            assert_eq!(
                settings.theme().unwrap(),
                &Theme {
                    logo: Some(Logo {
                        src: "_assets/logo.svg".into(),
                        src_dark: Some("_assets/logo-dark.svg".into()),
                    }),
                    radius: Radius::Large,
                    favicon: Some(Favicon {
                        src: "_assets/favicon.svg".into()
                    }),
                    colors: ColorsV2::default(),
                    color_mode: ColorMode::default(),
                }
            );
            assert_eq!(
                settings.open_api(),
                &[OpenApi {
                    spec_file: "/path/to/spec.json".into(),
                    uri_prefix: "/bobby".to_string(),
                    experimental: false
                }]
            );
            assert_eq!(settings.styles(), &[PathBuf::from("_assets/style.css")]);
            assert_eq!(
                settings.redirects(),
                &[Redirect {
                    from: "/foo".to_owned(),
                    to: "/bar".to_owned()
                }]
            );
            assert_eq!(
                settings.footer(),
                Some(&Footer {
                    links: vec![
                        FooterLink::External(ExternalLink {
                            label: "Terms of Service".to_string(),
                            external: "http://www.example.com".to_string(),
                        }),
                        FooterLink::External(ExternalLink {
                            label: "Privacy Policy".to_string(),
                            external: "http://www.example.com".to_string(),
                        })
                    ],
                    twitter: Some("Doctave".to_owned()),
                    linkedin: Some("Doctave".to_owned()),
                    github: Some("Doctave".to_owned()),
                    discord: None,
                    facebook: None,
                })
            );
        }

        #[test]
        fn v2_defaults() {
            let input = indoc! {r##"
            ---
                title: Acme Inc
        "##};

            let settings = Settings::parse(input).unwrap();

            assert!(true);
            assert_eq!(
                settings.footer(),
                Some(&Footer {
                    links: vec![],
                    twitter: None,
                    linkedin: None,
                    github: None,
                    discord: None,
                    facebook: None,
                })
            );
            assert_eq!(settings.redirects(), &[]);
            assert_eq!(
                settings.theme().unwrap(),
                &Theme {
                    logo: None,
                    radius: Radius::Medium,
                    favicon: None,
                    colors: ColorsV2::default(),
                    color_mode: ColorMode::default(),
                }
            );
        }

        #[test]
        fn v2_deserializes_tab_descriptions_from_tabs_key() {
            // This test is to ensure that the `tab_descriptions` is deserialized correctly
            // from "tabs" key
            let input = indoc! {r##"
            ---
            title: Acme Inc
            tabs:
              - label: "Tab 1"
                path: "/tab1"
              - label: "Tab 2"
                path: "/tab2"
            "##};

            let settings = Settings::parse(input).unwrap();

            assert_eq!(settings.tab_descriptions.len(), 2);
            assert_eq!(settings.tab_descriptions[0].label, "Tab 1");
            assert_eq!(settings.tab_descriptions[1].label, "Tab 2");
        }

        #[test]
        fn color_mode_settings_get_set() {
            let input = indoc! {r##"
            ---
            title: Acme Inc

            theme:
              color_mode: "dark"
            "##};

            let settings: Settings = serde_yaml::from_str(input).unwrap();

            assert_eq!(settings.theme.color_mode, ColorMode::Dark);
        }

        #[test]
        fn color_mode_settings_get_set_default() {
            let input = indoc! {r##"
            ---
            title: Acme Inc
            "##};

            let settings: Settings = serde_yaml::from_str(input).unwrap();

            assert_eq!(settings.theme.color_mode, ColorMode::Auto);
        }

        #[test]
        fn invalid_color_mode() {
            let input = indoc! {r##"
            ---
            title: Acme Inc

            theme:
              color_mode:"kettle
            "##};

            let err = Settings::parse(input).unwrap_err();
            println!("{:#?}", err);
            assert_eq!(err.description, "There was an error parsing your docapella.yaml:\n\ntheme: invalid type: string \"color_mode:\\\"kettle\", expected struct Theme at line 5 column 3");
        }
    }
}
