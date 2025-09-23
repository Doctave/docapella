#[macro_use]
extern crate indoc;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde;

pub use serde_json;

pub mod breadcrumb;
pub mod content_api;
mod description_extractor;
mod error_options;
mod frontmatter;
pub mod icon;
pub mod markdown;
pub mod markdown_navigation;
pub mod markdown_page;
pub mod navigation;
pub mod open_api;
pub mod open_api_page;
pub mod page_handle;
mod page_kind;
pub mod project;
mod render_context;
mod render_options;
pub mod renderer;
mod search_index;
pub mod settings;
mod slug;
pub mod structure;
pub mod structure_v2;
mod utils;
pub mod vale;

pub(crate) use markdown_page::MarkdownPage;
pub(crate) use open_api_page::OpenApiPage;

pub use renderable_ast::{Node as AstNode, ProseStatistics};

pub use content_api::{ContentApiResponse, ResponseContext};
pub use description_extractor::DescriptionExtractor;

pub use page_handle::PageHandle;
pub use page_kind::Ast;
pub use project::{InputContent, InputFile, Project};

pub use error_options::ErrorOptions;
pub use render_options::RenderOptions;

pub use shared_ast::{Point, Position};

pub use search_index::SearchIndex;

use markdown::*;

pub use structure::Structure;
pub use structure::SubTab;
pub use structure::Tab;

pub use markdown::autocomplete::{CompletionItem, CompletionItemKind};

use std::path::{Path, PathBuf};

pub const NAVIGATION_FILE_NAME: &str = "navigation.yaml";
pub const STRUCTURE_FILE_NAME: &str = "structure.yaml";
pub const DEPRECATED_NAVIGATION_FILE_NAME: &str = "_Navigation.md";
pub const SETTINGS_FILE_NAME: &str = "docapella.yaml";

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, PartialOrd, Ord)]
pub struct Error {
    pub code: usize,
    pub message: String,
    pub description: String,
    pub file: Option<PathBuf>,
    pub position: Option<Position>,
}

impl Error {
    pub const MISSING_DOCTAVE_YAML: usize = 10;
    pub const INVALID_DOCTAVE_YAML: usize = 11;
    pub const INVALID_STRUCTURE_YAML: usize = 12;
    pub const EMPTY_PROJECT: usize = 13;
    pub const MISSING_ROOT_README: usize = 20;
    pub const MISSING_NAVIGATION: usize = 30;
    pub const INVALID_NAVIGATION: usize = 31;
    pub const INVALID_STRUCTURE: usize = 32;
    pub const INVALID_REDIRECT: usize = 33;
    pub const IO_ERROR: usize = 40;
    pub const OPENAPI_REFERENCE: usize = 50;
    pub const INVALID_OPENAPI_SPEC: usize = 51;
    pub const LIQUID_TEMPLATE_ERROR: usize = 60;
    pub const OPENAPI_TEMPLATE_ERROR: usize = 70;
    pub const NAVIGATION_ERROR: usize = 80;
    pub const INVALID_FRONTMATTER: usize = 90;
    pub const BROKEN_INTERNAL_LINK: usize = 100;
    pub const INVALID_MARKDOWN_TEMPLATE: usize = 110;
    pub const INVALID_EXPRESSION: usize = 120;
    pub const INVALID_COMPONENT: usize = 130;
    pub const INVALID_CONDITIONAL: usize = 140;
    pub const INVALID_TABS: usize = 150;
    pub const INVALID_STEPS: usize = 160;
    pub const INVALID_OPENAPI_SCHEMA: usize = 170;
    pub const VALE_ERROR: usize = 180;

    fn in_file(&mut self, path: &Path) {
        self.file = Some(path.to_owned());
    }

    fn from_serde_yaml(
        serde_error: serde_yaml::Error,
        code: usize,
        message: String,
        file: Option<PathBuf>,
    ) -> Self {
        Error {
            code,
            message,
            file,
            position: None,
            description: format!("{}", serde_error),
        }
    }
}

impl From<std::io::Error> for crate::Error {
    fn from(other: std::io::Error) -> Self {
        Error {
            code: Self::IO_ERROR,
            message: "IO error occurred".to_owned(),
            description: format!("{}", other),
            file: None,
            position: None,
        }
    }
}

impl From<liquid::Error> for crate::Error {
    fn from(other: liquid::Error) -> Self {
        Error {
            code: Self::LIQUID_TEMPLATE_ERROR,
            message: "Error parsing liquid template".to_string(),
            description: format!("{}", other),
            file: None,
            position: None,
        }
    }
}

/// Conver a filesystem path to its URI path.
pub fn fs_to_uri_path(fs_path: &Path) -> String {
    let mut fs_path = fs_path.to_owned();

    fs_path.set_extension("");

    if fs_path.ends_with("README") {
        fs_path.pop();
    }
    if fs_path.starts_with("/") {
        fs_path = fs_path.strip_prefix("/").unwrap().to_owned();
    }

    let mut out = String::new();
    for part in fs_path.components() {
        out.push('/');

        if let std::path::Component::Normal(str_part) = part {
            out.push_str(&slug::slugify(str_part.to_string_lossy().as_ref()));
        }
    }

    if out.is_empty() {
        "/".to_string()
    } else {
        out
    }
}

/// Best guess of converting a URI path to its equivalent FS path.
///
/// The reason this is "best guess" is that going the other way
/// we convert non-ascii characters to slugs, so there isn't a total
/// ordering between FS and URI paths.
///
/// This is currently used by the desktop to provide a nice error
/// message saying "404 - put a file _here_ to render this path".
pub fn uri_to_fs_path(uri_path: &str) -> PathBuf {
    match uri_path {
        "/" | "" => PathBuf::from("README.md"),
        _ => {
            let mut path = uri_path
                .split('/')
                .fold(PathBuf::new(), |mut path, segment| {
                    path.push(segment);
                    path
                });

            path.set_extension("md");
            path
        }
    }
}

/// Convenience method for returning an AST from a Markdown section. This is
/// used by Jaleo to extract description information out of OpenAPI description
/// fields, which can be valid Markdown.
///
/// TODO(Nik): Surely this context cannot be enough? How does the AST know which
/// relative URL to use for its links?
pub fn markdown_to_ast(input: &str, opts: Option<&RenderOptions>) -> Result<Node> {
    let mut ctx = render_context::RenderContext::new();
    ctx.with_maybe_options(opts);

    markdown::ast(input, &ctx)
}

pub fn rewrite_links(content: &str, opts: Option<&RenderOptions>) -> String {
    let mut result = content.to_owned();

    if let Some(opts) = opts {
        for (from, to) in &opts.link_rewrites {
            if result.contains(from) {
                result = result.replace(from, to);
            }
        }
    }

    result
}

pub fn pretty_language_name(lang: &str) -> String {
    match lang.to_lowercase().as_str() {
        "php" => "PHP".to_string(),
        "csharp" => "C#".to_string(),
        "node" => "Node".to_string(),
        "js" => "JavaScript".to_string(),
        "javascript" => "JavaScript".to_string(),
        other => {
            let mut c = other.chars();
            match c.next() {
                None => String::new(),
                Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use pretty_assertions::assert_eq;
    use std::collections::HashMap;
    use std::path::{Path, PathBuf};

    #[test]
    fn rewrite_links_in_content() {
        let opts = RenderOptions {
            link_rewrites: HashMap::from([("/foo".to_owned(), "/bar".to_owned())]),
            ..Default::default()
        };

        assert_eq!(
            rewrite_links("Click [here](/foo)", Some(&opts)),
            "Click [here](/bar)"
        );
    }

    #[test]
    fn rewrite_links_multiple_in_content() {
        let opts = RenderOptions {
            link_rewrites: HashMap::from([
                ("/foo".to_owned(), "/bar".to_owned()),
                ("/fizz".to_owned(), "/buzz".to_owned()),
            ]),
            ..Default::default()
        };

        assert_eq!(
            rewrite_links("Click [here](/foo). And [here](/fizz)", Some(&opts)),
            "Click [here](/bar). And [here](/buzz)"
        );
    }

    #[test]
    fn get_page_by_uri_path() {
        let navigation = indoc! {"
        * [Root](README.md)
        "};
        let root = indoc! {"
        # I am Root!
        "};

        let files = vec![
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text(navigation.to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(root.to_owned()),
            },
            InputFile {
                path: PathBuf::from("Foo.md"),
                content: InputContent::Text(root.to_owned()),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let ast = project
            .get_page_by_uri_path("/")
            .expect("Could not find root page")
            .ast(None)
            .unwrap();

        assert!(ast
            .as_markdown()
            .unwrap()
            .inner_text()
            .contains("I am Root!"));

        assert!(project.get_page_by_uri_path("/Foo").is_some());
    }

    #[test]
    fn fs_to_uri_path_conversion() {
        assert_eq!(fs_to_uri_path(Path::new("README.md")), "/".to_string());
        assert_eq!(
            fs_to_uri_path(Path::new("foo/README.md")),
            "/foo".to_string()
        );
        assert_eq!(fs_to_uri_path(Path::new("Bar.md")), "/Bar".to_string());
    }

    #[test]
    fn converts_a_nested_readme_file_into_a_uri_without_a_trailing_slash() {
        let mut fs_path = PathBuf::new();
        fs_path.push("foo");
        fs_path.push("README.md");

        assert_eq!(&fs_to_uri_path(&fs_path), "/foo")
    }

    #[test]
    fn handles_non_url_safe_filenames() {
        let mut fs_path = PathBuf::new();
        fs_path.push("Æúű");
        fs_path.push("Exämplé.md");

        assert_eq!(&fs_to_uri_path(&fs_path), "/AEuu/Example")
    }

    #[test]
    fn uri_to_fs_path_conversion() {
        assert_eq!(&uri_to_fs_path("/"), &Path::new("README.md"));
        assert_eq!(&uri_to_fs_path("/foo"), &Path::new("foo.md"));
        assert_eq!(&uri_to_fs_path("/foo/bar"), &Path::new("foo/bar.md"));
    }

    #[test]
    fn from_file_list() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("frontend/info.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# README")),
            },
            InputFile {
                path: PathBuf::from("backend/info.md"),
                content: InputContent::Text(String::from("# Backend")),
            },
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from("* [Root](README.md)")),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        assert!(project.get_page_by_uri_path("/").is_some());
    }

    #[test]
    fn projects_can_verify_they_have_a_root_readme_file() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("frontend/info.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from("* [Frontend](frontend/info.md)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let opts = RenderOptions::default();
        let errors = project.verify(Some(&opts), None).unwrap_err();

        let readme_error = &errors[0];
        assert_eq!(
            &readme_error.message,
            r#"Missing root README.md. Add a file at "/README.md"."#
        )
    }

    #[test]
    fn projects_can_verify_default_user_preference_included_in_value_list() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# README")),
            },
            InputFile {
                path: PathBuf::from("frontend/info.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                - heading: "Guides"
                  items:
                  - label: "/Meh.md"
                    href: "/Meh.md"
                "#}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                title: Foo

                user_preferences:
                  plan:
                    label: Plan
                    default: Not in list
                    values:
                      - Starter
                      - Growth
                      - Enterprise

                "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let opts = RenderOptions::default();
        let errors = project.verify(Some(&opts), None).unwrap_err();

        let readme_error = &errors[0];
        assert_eq!(
            &readme_error.message,
            "User preference \"plan\" default value not in list of possible values",
        );
        assert_eq!(
            &readme_error.description,
            "Expected default value to be one of \"Starter\", \"Growth\", \"Enterprise\".\nFound \"Not in list\"."
        );
    }

    #[test]
    fn no_navigation_error() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# README")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let opts = RenderOptions::default();
        let errors = project.verify(Some(&opts), None).unwrap_err();
        let error = &errors[0];

        assert_eq!(error.code, Error::MISSING_NAVIGATION);
    }

    #[test]
    fn no_doctave_yaml_error() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# Content")),
            },
            InputFile {
                path: PathBuf::from("_Navigation.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
        ];

        let errors = Project::from_file_list(file_list).unwrap_err();
        let error = errors.first().unwrap();

        assert_eq!(error.code, Error::MISSING_DOCTAVE_YAML);
        assert_eq!(&error.message, "Missing docapella.yaml");
    }

    #[test]
    fn it_returns_open_api_spec_pages() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("frontend/info.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {"
                ---
                title: Foo
                open_api:
                    - spec_file: spec.json
                      uri_prefix: /test-api
                "}
                    .to_string(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# README")),
            },
            InputFile {
                path: PathBuf::from("spec.json"),
                content: InputContent::Text(String::from(include_str!(
                    "./../examples/open_api_specs/petstore.json"
                ))),
            },
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from("* [Root](README.md)")),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        assert!(project.get_page_by_uri_path("/test-api/pets").is_some());
        // Don't add the spec as a page
        assert!(project.get_page_by_uri_path("/spec").is_none());
    }

    #[test]
    fn deprecated_navigation_file_markdown_parser_opts() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("Meh.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from("# Frontend\n\n* [Meh](/Meh.md)")),
            },
        ];

        let opts = RenderOptions {
            prefix_link_urls: Some("/foo".to_string()),
            webbify_internal_urls: true,
            ..Default::default()
        };

        let project = Project::from_file_list(file_list).unwrap();
        assert_eq!(
            project.root_navigation(Some(&opts)).unwrap()[0].items[0].href(),
            Some("/foo/Meh")
        );
    }

    #[test]
    fn navigation_file_markdown_parser_opts() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("Meh.md"),
                content: InputContent::Text(String::from("# Frontend")),
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
              - label: "/Meh.md"
                href: "/Meh.md"
            "#})),
            },
        ];

        let opts = RenderOptions {
            prefix_link_urls: Some("/foo".to_string()),
            webbify_internal_urls: true,
            ..Default::default()
        };

        let project = Project::from_file_list(file_list).unwrap();
        assert_eq!(
            project.root_navigation(Some(&opts)).unwrap()[0].items[0].href(),
            Some("/foo/Meh")
        );
    }

    #[test]
    fn uses_modern_navigation_if_it_is_available_over_old_one() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("Meh.md"),
                content: InputContent::Text(String::from("# Frontend")),
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
              - label: "New"
                href: "/Meh.md"
            "#})),
            },
            InputFile {
                path: PathBuf::from(DEPRECATED_NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from("# Guides\n\n* [Old](/Meh.md)")),
            },
        ];

        let opts = RenderOptions {
            prefix_link_urls: Some("/foo".to_string()),
            webbify_internal_urls: true,
            ..Default::default()
        };

        let project = Project::from_file_list(file_list).unwrap();
        assert_eq!(
            project.root_navigation(Some(&opts)).unwrap()[0].items[0].label(),
            "New"
        );
    }

    #[test]
    fn returns_errors_about_the_navigation_yaml() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# Readme")),
            },
            InputFile {
                path: PathBuf::from("Meh.md"),
                content: InputContent::Text(String::from("# Frontend")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - invalid: "Not right"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let opts = RenderOptions::default();
        let errors = project.verify(Some(&opts), None).unwrap_err();

        let readme_error = &errors[0];
        assert_eq!(&readme_error.message, "Invalid navigation.yaml")
    }

    #[test]
    fn it_ignores_non_asset_binary_files() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# Readme")),
            },
            InputFile {
                path: PathBuf::from("Meh.md"),
                content: InputContent::Binary("".to_string()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let opts = RenderOptions::default();
        assert_eq!(project.verify(Some(&opts), None), Ok(()));
    }

    #[test]
    fn it_expands_relative_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("# Readme")),
            },
            InputFile {
                path: PathBuf::from("foo/baz.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[an link](./baz.md)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let opts = RenderOptions::default();

        let page = project.get_page_by_uri_path("/foo/bar").unwrap();
        let ast = page.ast(Some(&opts)).unwrap();

        let markdown_ast = ast.as_markdown().unwrap();
        let link_node = &markdown_ast.children[0].children[0];
        match &link_node.kind {
            NodeKind::Link { url, .. } => {
                assert_eq!(url, "/foo/baz.md", "Link was not relative");
            }
            _ => panic!("Not a link: {:#?}", link_node),
        }
    }

    #[test]
    fn it_checks_for_broken_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("[bad link](does_not_exist.md)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Broken link detected");
        assert!(
            errors[0].description.contains("does_not_exist.md"),
            "Bad link not found in error description"
        );
    }

    #[test]
    fn it_checks_for_broken_asset_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("![bad link](does_not_exist.png)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Broken asset link detected");
        assert!(
            errors[0].description.contains("does_not_exist.png"),
            "Bad link not found in error description"
        );
    }

    #[test]
    fn it_checks_for_broken_asset_links_html() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from(
                    r#"<a download href="/_assets/foo/bar.csv">Bar</a>"#,
                )),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                ---
                title: An Project
                "# })),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Broken asset link detected");
        assert!(
            errors[0].description.contains("/_assets/foo/bar.csv"),
            "Bad link not found in error description"
        );
    }

    #[test]
    fn it_checks_for_broken_links_in_components_v2() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! { r#"
                    <Button href="does_not_exist.md">Click me</Button>
                    "# }
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
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Broken link detected");
        assert!(
            errors[0].description.contains("does_not_exist.md"),
            "Bad link not found in error description"
        );
    }

    #[test]
    fn it_checks_for_broken_links_with_all_user_preferences() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from(indoc! {"
                    {% if DOCTAVE.user_preferences.type == 'advanced' %}
                    [bad link](does_not_exist.md)
                    {% else %}
                    [good link](README.md)
                    {% endif %}
                "})),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                    ---
                    title: An Project
                    user_preferences:
                      type:
                        label: Type
                        default: basic
                        values:
                          - basic
                          - advanced
                "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Broken link detected");
        assert!(
            errors[0].description.contains("does_not_exist.md"),
            "Bad link not found in error description"
        );
    }

    #[test]
    fn it_checks_for_broken_links_in_nav_with_all_user_preferences() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::new()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                    ---
                    title: An Project
                    user_preferences:
                      type:
                        label: Type
                        default: basic
                        values:
                          - basic
                          - advanced
                "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                - heading: "Guides"
                  items:
                    - label: Broken
                      href: /nope.md
                      show_if:
                        user_preferences:
                          type:
                            equals: advanced
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors[0].message, "Broken link detected in navigation");
        assert!(
            errors[0].description.contains("nope.md"),
            "Bad link not found in error description"
        );
    }

    #[test]
    fn it_does_not_think_relative_links_are_broken_if_they_resolve_to_a_correct_file() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("foo/bar.md"),
                content: InputContent::Text(String::from("[good link](./baz.md)")),
            },
            InputFile {
                path: PathBuf::from("foo/baz.md"),
                content: InputContent::Text(String::from("[bad link](./fizz.md)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(errors.len(), 1);

        assert_eq!(errors[0].message, "Broken link detected");
        assert!(errors[0].description.contains("./fizz.md"));
    }

    #[test]
    fn it_does_not_think_external_links_are_broken_internal_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from(
                    "[OpenAPI](https://swagger.io/specification/)",
                )),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        if let Err(errors) = project.verify(None, None) {
            if errors.iter().any(|e| e.message == "Broken link detected") {
                panic!("External link interpreted as broken internal link");
            }
            panic!("Unexpcted error verifying project");
        }
    }

    #[test]
    fn it_detects_redirects_for_broken_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("[bad link](does_not_exist.md)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                ---
                title: An Project
                redirects:
                  - from: /does_not_exist
                    to: /
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None);

        assert!(errors.is_ok(), "Redirect not detected for broken link");
    }

    #[test]
    fn it_verifies_links_in_the_navigation_structure() {
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
                - href: /Tutorial.md
                  label: Tutorial
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project
            .verify(None, None)
            .expect_err("Project was valid with a broken nav link");

        assert_eq!(errors.len(), 1);

        assert_eq!(errors[0].message, "Broken link detected in navigation");
        assert!(errors[0].description.contains("/Tutorial.md"));
    }

    #[test]
    fn it_detects_redirects_for_broken_links_in_navigation() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("Hello world")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                ---
                title: An Project
                redirects:
                  - from: /does_not_exist
                    to: /
                "#})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
              items:
                - href: /does_not_exist
                  label: Tutorial
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project.verify(None, None);

        assert!(
            errors.is_ok(),
            "Redirect not detected for broken link for nav"
        );
    }

    #[test]
    fn it_does_not_flag_external_links_in_navigation_structure() {
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
                - external: https://www.example.com
                  label: External
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        if let Err(errors) = project.verify(None, None) {
            if errors
                .iter()
                .any(|e| e.message == "Broken link detected in navigation")
            {
                panic!("External link interpreted in nav as broken internal link");
            }
            panic!("Unexpcted error verifying project");
        }
    }

    #[test]
    fn it_verifies_links_in_openapi_pages() {
        let spec_str = include_str!("../examples/open_api_specs/broken_internal_link.json");

        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(String::from(spec_str)),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                        ---
                        title: An Project
                        open_api:
                          - spec_file: openapi.json
                            uri_prefix: /api
                        "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        let errors = project
            .verify(None, None)
            .expect_err("Project was valid with a broken openapi internal link");

        assert_eq!(errors.len(), 1);

        assert_eq!(errors[0].message, "Broken link in OpenAPI spec");
        assert!(errors[0].description.contains("/bad/link.md"));
        assert!(errors[0].description.contains("openapi.json"));
    }

    #[test]
    fn it_shows_open_api_navigation_if_show_if_matches_default() {
        let spec_str = include_str!("../examples/open_api_specs/petstore.json");

        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(String::from(spec_str)),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                        ---
                        title: An Project
                        open_api:
                          - spec_file: openapi.json
                            uri_prefix: /api

                        user_preferences:
                          plan:
                            label: Plan
                            default: Starter
                            values:
                              - Starter
                              - Enterprise
                        "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
                - heading: "Guides"
                  items:
                    - open_api_spec: openapi.json
                      show_if:
                        user_preferences:
                          plan:
                            equals: Starter
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        project.verify(None, None).expect("Invalid project");

        let nav = project
            .root_navigation(Some(&RenderOptions::default()))
            .expect("could not build nav");
        assert_eq!(
            nav.sections[0].items.len(),
            2,
            "Nav not set with default user preference"
        );
    }

    #[test]
    fn it_handles_relative_links_in_openapi_specs() {
        let spec_str = include_str!("../examples/open_api_specs/relative_internal_link.json");

        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("api/foo.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(String::from(spec_str)),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                        ---
                        title: An Project
                        open_api:
                          - spec_file: openapi.json
                            uri_prefix: /api
                        "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        if let Err(errors) = project.verify(None, None) {
            if errors.iter().any(|e| e.message.contains("Broken link")) {
                panic!("External link interpreted in OpenAPI spec as broken internal link");
            }
            panic!("Unexpcted error verifying project");
        }
    }

    #[test]
    fn it_does_not_flag_openapi_links_in_navigation() {
        let spec_str = include_str!("../examples/open_api_specs/petstore.json");

        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(String::from(spec_str)),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {"
                    ---
                    title: An Project

                    open_api:
                      - spec_file: openapi.json
                        uri_prefix: /api
                "}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
              items:
              - label: Pets
                href: /api/pets

            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        if let Err(errors) = project.verify(None, None) {
            if errors
                .iter()
                .any(|e| e.message == "Broken link detected in navigation")
            {
                panic!("OpenAPI link interpreted in nav as broken link");
            }
            panic!("Unexpcted error verifying project");
        }
    }

    #[test]
    fn generates_an_overview_page_for_the_openapi_spec() {
        let spec_str = include_str!("../examples/open_api_specs/petstore.json");

        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("openapi.json"),
                content: InputContent::Text(String::from(spec_str)),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                        ---
                        title: An Project
                        open_api:
                          - spec_file: openapi.json
                            uri_prefix: /api
                        "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
              items:
                - open_api_spec: openapi.json
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        let page = project
            .get_page_by_uri_path("/api")
            .expect("API overview page not found");

        let html = page.ast(None).unwrap();
        let inner_text = html.as_markdown().unwrap().inner_text();

        assert!(inner_text.contains("Swagger Petstore"));
        assert!(inner_text.contains("1.0.0"));
        assert!(inner_text.contains("An markdown description"));

        let nav = project.root_navigation(None).unwrap();

        assert!(
            nav.has_link_to("/api"),
            "API overview link missing in navigation"
        );
    }

    #[test]
    fn it_renders_partials_in_markdown_files_when_loaded() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from(
                    "{% include \"_partials/example.md\" name: \"Alice\" %}",
                )),
            },
            InputFile {
                path: if cfg!(windows) {
                    PathBuf::from("_partials\\example.md")
                } else {
                    PathBuf::from("_partials/example.md")
                },
                content: InputContent::Text(String::from("Hello, {{ name }}")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {"
                        ---
                        title: An Project
                        "})),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();

        let page = project.get_page_by_uri_path("/").unwrap();
        assert_eq!(
            page.ast(Some(&RenderOptions::default()))
                .unwrap()
                .as_markdown()
                .unwrap()
                .inner_text(),
            "Hello, Alice"
        );
    }

    #[test]
    fn returns_navigation_file_based_on_subtab() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: PathBuf::from("tab1/Meh.md"),
                content: InputContent::Text("# Hi".to_string()),
            },
            InputFile {
                path: format!("tab1/{}", NAVIGATION_FILE_NAME).into(),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                - heading: Subtab1 Nav
                  items:
                    - label: "Meh.md"
                      href: "Meh.md"
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
            InputFile {
                path: PathBuf::from(STRUCTURE_FILE_NAME),
                content: InputContent::Text(
                    indoc! {r#"
                ---
                tabs:
                  - label: Default
                    subtabs:
                      - label: Default
                        path: /
                  - label: Tab1
                    subtabs:
                      - label: Subtab1
                        path: /tab1/
                "#}
                    .to_string(),
                ),
            },
        ];

        let opts = RenderOptions {
            prefix_link_urls: Some("/v1".to_string()),
            ..Default::default()
        };

        let project = Project::from_file_list(file_list).unwrap();
        assert!(project.root_navigation(Some(&opts)).is_ok());
        assert_eq!(
            project.navigation(Some(&opts), "/tab1/").unwrap()[0].items[0].href(),
            Some("/v1/tab1/Meh.md")
        );
    }

    #[test]
    fn bug_doc_795_dots_in_folders_causing_broken_links() {
        let file_list = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from("v.1.5-to-v.1.6.0/foo.md"),
                content: InputContent::Text(String::from("[good link](./bar.md)")),
            },
            InputFile {
                path: PathBuf::from("v.1.5-to-v.1.6.0/bar.md"),
                content: InputContent::Text(String::from("[good link](./foo.md)")),
            },
            InputFile {
                path: PathBuf::from("v.1.5-to-v.1.6.0/README.md"),
                content: InputContent::Text(String::from("[good link](./foo.md)")),
            },
            InputFile {
                path: PathBuf::from("v.1.5-to-v.1.6.0/other.md"),
                content: InputContent::Text(String::from("[good link](./nested/bar.md)")),
            },
            InputFile {
                path: PathBuf::from("v.1.5-to-v.1.6.0/nested/bar.md"),
                content: InputContent::Text(String::from("[good link](../foo.md)")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(String::from("---\ntitle: An Project")),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(String::from(indoc! {r#"
            - heading: "Guides"
            "#})),
            },
        ];

        let project = Project::from_file_list(file_list).unwrap();
        assert_eq!(project.verify(None, None), Ok(()));
    }
}
