use crate::markdown::primitive_components;
use crate::project::Project;
use crate::render_context::RenderContext;

use itertools::Itertools;
use regex::Regex;
use unix_path::{Component as UnixComponent, Path as UnixPath, PathBuf as UnixPathBuf};

use std::path::{Component, Path};

lazy_static! {
    static ref OPEN_LINK_URL_REGEX: Regex =
        Regex::new(r#"\[[^\]]*\]\((?<link>([^\)]*))$"#).unwrap();
    static ref OPEN_ASSET_LINK_URL_REGEX: Regex =
        Regex::new(r#"!\[[^\]]*\]\((?<link>([^\)]*))$"#).unwrap();
    static ref CODE_BLOCK_REGEX: Regex = Regex::new(r#"(`{3,}|~{3,})"#).unwrap();
    static ref COMPONENT_NAME_REGEX: Regex =
        Regex::new(r#"<(?<component>[A-Za-z0-9.]*)$"#).unwrap();
    static ref COMPONENT_ATTRIBUTE_NAME_REGEX: Regex = Regex::new(
        r#"<(?P<component>[A-Z][a-zA-Z0-9_\.]*)(?:[^>]*?[=\s])(?P<attribute>[^\s"'=<>]*)$"#
    )
    .unwrap();
    static ref COMPONENT_ATTRIBUTE_VALUE_REGEX: Regex = Regex::new(
        r#"<(?P<component>[A-Z][a-zA-Z0-9_\.]*)(?:[^>]*?)(?P<attribute>[^\s"'=<>]+)\s*=\s*(['"])(?P<value_prefix>.*)$"#
    )
    .unwrap();

    static ref PRIMITIVE_COMPONENT_AUTOCOMPLETES: Vec<Box<dyn PrimitiveComponentAutocomplete>> = vec![
        // Grid component
        Box::<primitive_components::Grid>::default(),
        // Flex component
        Box::<primitive_components::Flex>::default(),
        // CodeSelect component
        Box::<primitive_components::CodeSelect>::default(),
        // Box component
        Box::<primitive_components::CBox>::default(),
        // Step component
        Box::<primitive_components::Step>::default(),
        // Steps component
        Box::<primitive_components::Steps>::default(),
        // Tab component
        Box::<primitive_components::Tab>::default(),
        // Tabs component
        Box::<primitive_components::Tabs>::default(),
    ];
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct CompletionItem {
    pub label: String,
    pub kind: CompletionItemKind,
    pub insert_text: String,
    pub trigger_completion: bool,
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub enum CompletionItemKind {
    File,
    Class,
    Field,
    Enum,
}

/// Returns the completion items at the end of the given string.
///
/// First determines the context of the string, figuring out if it's a link
/// url, an asset link url, a component name, or a component attribute value.
///
/// Then calls the appropriate autocomplete function for that context to
/// return the completion items.
pub(crate) fn autocomplete(
    markdown: &str,
    fs_path: &Path,
    project: &Project,
    ctx: &RenderContext,
) -> Vec<CompletionItem> {
    let context = get_autocomplete_context(markdown, ctx);

    match context {
        AutocompleteContext::LinkUrl(url) => autocomplete_link_url(&url, fs_path, project),
        AutocompleteContext::AssetLinkUrl(url) => autocomplete_asset_link_url(&url, project),
        AutocompleteContext::ComponentName(name) => autocomplete_component_name(&name, ctx),
        AutocompleteContext::ComponentAttributeName(component, attribute) => {
            autocomplete_component_attribute_name(&component, &attribute, ctx)
        }
        AutocompleteContext::ComponentAttributeValue(component, attribute, value_prefix) => {
            autocomplete_component_attribute_value(&component, &attribute, &value_prefix, ctx)
        }
        _ => vec![],
    }
}

fn autocomplete_asset_link_url(url_prefix: &str, project: &Project) -> Vec<CompletionItem> {
    /// Multi-platform helper to convert a path to a URL path that guarantees
    /// we always have '/' as the path separator.
    fn asset_path_to_url_path(path: &Path) -> String {
        let mut out = String::new();

        for component in path.components() {
            if let std::path::Component::Normal(c) = component {
                out.push('/');
                out.push_str(&c.to_string_lossy())
            }
        }

        out
    }

    project
        .assets
        .iter()
        .filter(|asset| {
            let absolute_asset_path = asset_path_to_url_path(&asset.path);

            absolute_asset_path.starts_with(url_prefix)
        })
        .map(|asset| {
            let absolute_asset_path = asset_path_to_url_path(&asset.path);

            CompletionItem {
                label: absolute_asset_path.to_string(),
                kind: CompletionItemKind::File,
                insert_text: format!(
                    "{})",
                    absolute_asset_path
                        .strip_prefix(url_prefix)
                        .unwrap_or(&absolute_asset_path)
                ),
                trigger_completion: false,
            }
        })
        .collect()
}

fn autocomplete_link_url(
    url_prefix: &str,
    base_path: &Path,
    project: &Project,
) -> Vec<CompletionItem> {
    let absolute_url_prefix = if url_prefix.starts_with('/') || url_prefix.is_empty() {
        url_prefix.to_string()
    } else {
        // First, we need to convert the url_prefix into an absolute path
        // based on the base_path, which represents the file that the
        // autocomplete is being run in.
        //
        // This is only used for the comparison to find matching files.
        let mut base = unix_path::PathBuf::new();

        for component in base_path.parent().unwrap_or(base_path).components() {
            if let std::path::Component::Normal(c) = component {
                base.push(&*c.to_string_lossy())
            }
        }

        let parse_prefix = unix_path::Path::new(url_prefix);
        for component in parse_prefix.components() {
            match component {
                unix_path::Component::Normal(c) => base.push(&*c.to_string_lossy()),
                unix_path::Component::ParentDir => {
                    base.pop();
                }
                _ => {}
            }
        }

        // This is now the absolute path in the project that we can use to
        // compare against the paths of the pages.
        format!("/{}", base.display())
    };

    project
        .pages()
        .iter()
        .filter_map(|a| {
            if a.fs_path().starts_with(&absolute_url_prefix)
                || a.uri_path().starts_with(&absolute_url_prefix)
            {
                // Now that we have match, if the prefix started with a `./` or `../`
                // we need to keep that in the label and the insert text.
                //
                // So e.g if the original url_prefix was `./f`, we're in a directory
                // called "nested", and we've matched a page at `/nested/foo.md`
                // (uri_path would be `nested/foo`), we want to show the label as
                // `./foo.md` and the insert_text as `oo` (to complete the `./foo` part).

                let mut label = a.uri_path().to_string();

                // Remove the file extension for files other than README.md
                if !a
                    .fs_path()
                    .file_name()
                    .unwrap_or_default()
                    .to_str()
                    .unwrap_or("")
                    .eq_ignore_ascii_case("README.md")
                {
                    label = label
                        .rsplit_once('.')
                        .map(|(name, _)| name.to_string())
                        .unwrap_or(label);
                }

                if url_prefix.starts_with("./") || url_prefix.starts_with("../") {
                    // Handle relative path completion
                    let unixified_base_path = to_unix_path(base_path)?;
                    let fs_path = to_unix_path(a.fs_path())?;

                    let relative_base =
                        unixified_base_path.parent().unwrap_or(&unixified_base_path);
                    let relative_path = path_diff(&fs_path, relative_base)?;

                    let mut relative_label = relative_path.display().to_string();

                    // Remove file extension
                    relative_label = relative_label
                        .rsplit_once('.')
                        .map(|(name, ext)| {
                            if ext.eq_ignore_ascii_case("md") {
                                name.to_string()
                            } else {
                                relative_label.clone()
                            }
                        })
                        .unwrap_or(relative_label);

                    // Handle README.md special case
                    if a.fs_path()
                        .file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("")
                        .eq_ignore_ascii_case("README.md")
                    {
                        relative_label = relative_label.trim_end_matches("README").to_string();
                        if relative_label.ends_with('/') {
                            relative_label.pop();
                        }
                    }
                    if url_prefix.starts_with("./") {
                        label = format!("./{}", relative_label);
                    } else if url_prefix.starts_with("../") {
                        // We get this for free
                        label = relative_label.to_string();
                    }
                }

                let insert_text = label.strip_prefix(url_prefix).unwrap_or(&label);

                Some(crate::CompletionItem {
                    kind: crate::CompletionItemKind::File,
                    insert_text: insert_text.to_string(),
                    label,
                    trigger_completion: false,
                })
            } else {
                None
            }
        })
        .unique_by(|a| a.label.clone())
        .collect()
}
fn autocomplete_component_name(prefix: &str, ctx: &RenderContext) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for component in ctx.custom_components {
        if let Ok(title) = component.title() {
            if title.starts_with(prefix) {
                completion_items.push(CompletionItem {
                    label: title.to_string(),
                    kind: CompletionItemKind::Class,
                    insert_text: title.strip_prefix(prefix).unwrap_or(&title).to_string(),
                    trigger_completion: false,
                });
            }
        }
    }

    for component in PRIMITIVE_COMPONENT_AUTOCOMPLETES.iter() {
        if component.title().starts_with(prefix) {
            completion_items.push(CompletionItem {
                label: component.title().to_string(),
                kind: CompletionItemKind::Class,
                insert_text: component
                    .title()
                    .strip_prefix(prefix)
                    .unwrap_or(component.title())
                    .to_string(),
                trigger_completion: false,
            });
        }
    }

    completion_items
}

fn autocomplete_component_attribute_name(
    component_name: &str,
    attribute_prefix: &str,
    ctx: &RenderContext,
) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for component in ctx.custom_components {
        if component.matches_title(component_name) {
            if let Ok(comp) = component.build() {
                comp.attributes
                    .iter()
                    .filter(|a| a.title.starts_with(attribute_prefix))
                    .for_each(|a| {
                        completion_items.push(CompletionItem {
                            label: a.title.to_string(),
                            kind: CompletionItemKind::Field,
                            insert_text: format!(
                                "{}=\"",
                                a.title.strip_prefix(attribute_prefix).unwrap_or(&a.title)
                            ),
                            trigger_completion: true,
                        })
                    });
            }
        }
    }

    for component in PRIMITIVE_COMPONENT_AUTOCOMPLETES.iter() {
        if component.title() == component_name {
            for attribute in component.attributes() {
                if attribute.starts_with(attribute_prefix) {
                    completion_items.push(CompletionItem {
                        label: attribute.to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: format!(
                            "{}=\"",
                            attribute
                                .strip_prefix(attribute_prefix)
                                .unwrap_or(attribute)
                        ),
                        trigger_completion: true,
                    });
                }
            }
        }
    }

    completion_items
}

fn autocomplete_component_attribute_value(
    component_name: &str,
    attribute: &str,
    value_prefix: &str,
    ctx: &RenderContext,
) -> Vec<CompletionItem> {
    let mut completion_items = vec![];

    for component in ctx.custom_components {
        if component.matches_title(component_name) {
            if let Ok(comp) = component.build() {
                if let Some(attribute) = comp.attributes.iter().find(|a| a.title == attribute) {
                    for value in &attribute.validation.is_one_of {
                        if value.as_string().starts_with(value_prefix) {
                            completion_items.push(CompletionItem {
                                label: value.as_string(),
                                kind: CompletionItemKind::Enum,
                                insert_text: format!(
                                    "{}\"",
                                    value
                                        .as_string()
                                        .strip_prefix(value_prefix)
                                        .unwrap_or(&value.as_string())
                                ),
                                trigger_completion: false,
                            });
                        }
                    }
                }
            }
        }
    }

    for component in PRIMITIVE_COMPONENT_AUTOCOMPLETES.iter() {
        if component.title() == component_name {
            for attribute_name in component.attributes() {
                if attribute_name == attribute {
                    for value in component.attribute_values(attribute_name) {
                        completion_items.push(CompletionItem {
                            label: value.to_string(),
                            kind: CompletionItemKind::Enum,
                            insert_text: format!(
                                "{}\"",
                                value
                                    .to_string()
                                    .strip_prefix(value_prefix)
                                    .unwrap_or(value)
                            ),
                            trigger_completion: false,
                        });
                    }
                }
            }
        }
    }

    completion_items
}

fn path_diff(path: &UnixPath, base: &UnixPath) -> Option<UnixPathBuf> {
    if path.is_absolute() != base.is_absolute() {
        return if path.is_absolute() {
            Some(path.to_path_buf())
        } else {
            None
        };
    }

    let mut ita = path.components();
    let mut itb = base.components();
    let mut comps: Vec<UnixComponent> = vec![];
    loop {
        match (ita.next(), itb.next()) {
            (None, None) => break,
            (Some(a), None) => {
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
            (None, _) => comps.push(UnixComponent::ParentDir),
            (Some(a), Some(b)) if comps.is_empty() && a == b => (),
            (Some(a), Some(UnixComponent::CurDir)) => comps.push(a),
            (Some(_), Some(UnixComponent::ParentDir)) => return None,
            (Some(a), Some(_)) => {
                comps.push(UnixComponent::ParentDir);
                for _ in itb {
                    comps.push(UnixComponent::ParentDir);
                }
                comps.push(a);
                comps.extend(ita.by_ref());
                break;
            }
        }
    }
    Some(comps.into_iter().collect())
}

pub fn to_unix_path(path: &Path) -> Option<UnixPathBuf> {
    let mut unix_path = UnixPathBuf::new();

    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                // Windows-specific prefixes can't be converted
                return None;
            }
            Component::RootDir => {
                unix_path.push(UnixComponent::RootDir);
            }
            Component::CurDir => {
                unix_path.push(UnixComponent::CurDir);
            }
            Component::ParentDir => {
                unix_path.push(UnixComponent::ParentDir);
            }
            Component::Normal(os_str) => {
                if let Some(s) = os_str.to_str() {
                    unix_path.push(s);
                } else {
                    // Non-UTF8 path component
                    return None;
                }
            }
        }
    }

    Some(unix_path)
}

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
pub enum AutocompleteContext {
    None,
    /// Autocomplete suggestions for a link url.
    /// The string is the prefix of the link url that is being completed.
    LinkUrl(String),
    AssetLinkUrl(String),
    ComponentName(String),
    /// The first string is the component name, the second is the attribute prefix we're completing.
    ComponentAttributeName(String, String),
    /// The first string is the component name, the second is the attribute, and the third is the value prefix we're completing.
    ComponentAttributeValue(String, String, String),
}

/// Returns the autocomplete context at the end of the given string.
///
/// Uses a regexes as a quick and dirty way to determine the context.
/// This is what VSCode Markdown extension uses to determine the context,
/// so we aren't _worse_ than them. One day, do this better with a real
/// parser.
fn get_autocomplete_context(markdown: &str, _ctx: &RenderContext) -> AutocompleteContext {
    if is_in_code_block(markdown) {
        return AutocompleteContext::None;
    }

    if let Some(url) = is_end_of_asset_link_url(markdown) {
        AutocompleteContext::AssetLinkUrl(url.to_string())
    } else if let Some(url) = is_end_of_link_url(markdown) {
        AutocompleteContext::LinkUrl(url.to_string())
    } else if let Some(name) = is_in_component_name(markdown) {
        AutocompleteContext::ComponentName(name.to_string())
    } else if let Some((component, attribute_prefix)) = is_in_component_attribute_name(markdown) {
        AutocompleteContext::ComponentAttributeName(component, attribute_prefix)
    } else if let Some((component, attribute, value_prefix)) =
        is_in_component_attribute_value(markdown)
    {
        AutocompleteContext::ComponentAttributeValue(component, attribute, value_prefix)
    } else {
        AutocompleteContext::None
    }
}

/// Returns true if the given markdown string is in a code block.
///
/// Does this by checking if we have an uneven number of backticks or tildes
/// in the document
fn is_in_code_block(markdown: &str) -> bool {
    CODE_BLOCK_REGEX.find_iter(markdown).count() % 2 == 1
}

fn is_end_of_link_url(input: &str) -> Option<&str> {
    OPEN_LINK_URL_REGEX
        .captures(input)
        .and_then(|c| c.name("link").map(|m| m.as_str()))
}

fn is_end_of_asset_link_url(input: &str) -> Option<&str> {
    OPEN_ASSET_LINK_URL_REGEX
        .captures(input)
        .and_then(|c| c.name("link").map(|m| m.as_str()))
}

fn is_in_component_name(input: &str) -> Option<&str> {
    COMPONENT_NAME_REGEX
        .captures(input)
        .and_then(|c| c.name("component").map(|m| m.as_str()))
}

fn is_in_component_attribute_name(input: &str) -> Option<(String, String)> {
    COMPONENT_ATTRIBUTE_NAME_REGEX.captures(input).map(|caps| {
        let component = caps
            .name("component")
            .map_or("".to_string(), |m| m.as_str().to_string());
        let attribute = caps
            .name("attribute")
            .map_or("".to_string(), |m| m.as_str().to_string());
        (component, attribute)
    })
}

fn is_in_component_attribute_value(input: &str) -> Option<(String, String, String)> {
    COMPONENT_ATTRIBUTE_VALUE_REGEX.captures(input).map(|caps| {
        let component = caps
            .name("component")
            .map_or("".to_string(), |m| m.as_str().to_string());
        let attribute = caps
            .name("attribute")
            .map_or("".to_string(), |m| m.as_str().to_string());
        let value_prefix = caps
            .name("value_prefix")
            .map_or("".to_string(), |m| m.as_str().to_string());
        (component, attribute, value_prefix)
    })
}

pub(crate) trait PrimitiveComponentAutocomplete: Sync + Send {
    fn title(&self) -> &str;
    fn attributes(&self) -> Vec<&str>;
    fn attribute_values(&self, attribute: &str) -> Vec<&str>;
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn determines_context_for_link_url() {
        let markdown = "[link](./fo";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::LinkUrl("./fo".to_string()));
    }

    #[test]
    fn determines_context_for_link_url_2() {
        let markdown = "[link](";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::LinkUrl("".to_string()));
    }

    #[test]
    fn determines_context_for_link_url_3() {
        let markdown = "[link]";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::None);
    }

    #[test]
    fn determines_context_for_link_url_4() {
        let markdown = "[link](/url)";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::None);
    }

    #[test]
    fn determines_context_for_asset_url() {
        let markdown = "![asset](./fo";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(
            context,
            AutocompleteContext::AssetLinkUrl("./fo".to_string())
        );
    }

    #[test]
    fn determines_context_for_asset_url_2() {
        let markdown = "![asset](";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::AssetLinkUrl("".to_string()));
    }

    #[test]
    fn determines_context_for_asset_url_3() {
        let markdown = "![asset]";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::None);
    }

    #[test]
    fn determines_context_for_asset_url_4() {
        let markdown = "![asset](/url)";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::None);
    }

    #[test]
    fn ignores_autocompletion_in_code_block() {
        let markdown = "```\n[link](./fo";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::None);
    }

    #[test]
    fn ignores_autocompletion_in_code_block_2() {
        let markdown = "```rust\nthis is rust code\n```\n[link](./fo";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::LinkUrl("./fo".to_string()));
    }

    #[test]
    fn determines_context_for_component_name() {
        let markdown = "<Com";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(
            context,
            AutocompleteContext::ComponentName("Com".to_string())
        );
    }

    #[test]
    fn determines_context_for_component_name_2() {
        let markdown = "<Com.Pon";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(
            context,
            AutocompleteContext::ComponentName("Com.Pon".to_string())
        );
    }

    #[test]
    fn determines_context_for_component_name_3() {
        let markdown = "something <";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(context, AutocompleteContext::ComponentName("".to_string()));
    }

    #[test]
    fn determines_context_for_component_name_4() {
        let markdown = "<Component f";

        let ctx = RenderContext::new();
        let context = get_autocomplete_context(markdown, &ctx);

        assert_eq!(
            context,
            AutocompleteContext::ComponentAttributeName("Component".to_string(), "f".to_string())
        );
    }

    mod uri_link_autocomplete {
        use super::*;
        use crate::{
            InputContent, InputFile, RenderOptions, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
        };
        use std::path::PathBuf;

        #[test]
        fn gets_autocomplete_for_absolute_links() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("foo.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items =
                project.autocomplete("[link](/fo", Path::new("README.md"), Some(&render_opts));

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "/foo".to_string(),
                    kind: crate::CompletionItemKind::File,
                    insert_text: "o".to_string(),
                    trigger_completion: false,
                }]
            );
        }

        #[test]
        fn gets_autocomplete_for_relative_links() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/alice.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/bob.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "[link](./b",
                Path::new("nested/alice.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "./bob".to_string(),
                    kind: crate::CompletionItemKind::File,
                    insert_text: "ob".to_string(),
                    trigger_completion: false,
                }]
            );
        }

        #[test]
        fn gets_autocomplete_from_readme_files() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/alice.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items =
                project.autocomplete("[link](/nest", Path::new("README.md"), Some(&render_opts));

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "/nested/alice".to_string(),
                    kind: crate::CompletionItemKind::File,
                    insert_text: "ed/alice".to_string(),
                    trigger_completion: false,
                }]
            );
        }

        #[test]
        fn gets_autocomplete_from_readme_files_relative() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/alice.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "[link](./ali",
                Path::new("nested/README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "./alice".to_string(),
                    kind: crate::CompletionItemKind::File,
                    insert_text: "ce".to_string(),
                    trigger_completion: false,
                }]
            );
        }

        #[test]
        fn gets_autocomplete_from_parent_files() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/alice.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("bob.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "[link](../b",
                Path::new("nested/alice.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "../bob".to_string(),
                    kind: crate::CompletionItemKind::File,
                    insert_text: "ob".to_string(),
                    trigger_completion: false,
                }]
            );
        }

        #[test]
        fn empty_autocomplete_returns_absolute_links_from_root() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("nested/alice.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("bob.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items =
                project.autocomplete("[link](", Path::new("nested/alice.md"), Some(&render_opts));

            assert_eq!(
                completion_items,
                vec![
                    crate::CompletionItem {
                        label: "/".to_string(),
                        kind: crate::CompletionItemKind::File,
                        insert_text: "/".to_string(),
                        trigger_completion: false,
                    },
                    crate::CompletionItem {
                        label: "/nested/alice".to_string(),
                        kind: crate::CompletionItemKind::File,
                        insert_text: "/nested/alice".to_string(),
                        trigger_completion: false,
                    },
                    crate::CompletionItem {
                        label: "/bob".to_string(),
                        kind: crate::CompletionItemKind::File,
                        insert_text: "/bob".to_string(),
                        trigger_completion: false,
                    },
                ]
            );
        }
    }

    mod asset_link_autocomplete {
        use super::*;
        use crate::{
            InputContent, InputFile, RenderOptions, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
        };
        use std::path::PathBuf;

        #[test]
        fn gets_autocomplete_for_absolute_links() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("_assets/cat.jpg"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "![cat](/_assets/c",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "/_assets/cat.jpg".to_string(),
                    kind: crate::CompletionItemKind::File,
                    insert_text: "at.jpg)".to_string(),
                    trigger_completion: false,
                }]
            );
        }
    }

    mod component_name_autocomplete {
        use super::*;
        use crate::{
            InputContent, InputFile, RenderOptions, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
        };
        use std::path::PathBuf;

        #[test]
        fn gets_autocomplete_for_component_name() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Ca",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![
                    crate::CompletionItem {
                        label: "Card".to_string(),
                        kind: crate::CompletionItemKind::Class,
                        insert_text: "rd".to_string(),
                        trigger_completion: false,
                    },
                    crate::CompletionItem {
                        label: "Callout".to_string(),
                        kind: crate::CompletionItemKind::Class,
                        insert_text: "llout".to_string(),
                        trigger_completion: false,
                    }
                ]
            );
        }

        #[test]
        fn gets_autocomplete_for_custom_components() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("_components/Custom.md"),
                    content: InputContent::Text(String::from("This is a custom component")),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Component.",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![crate::CompletionItem {
                    label: "Component.Custom".to_string(),
                    kind: crate::CompletionItemKind::Class,
                    insert_text: "Custom".to_string(),
                    trigger_completion: false,
                },]
            );
        }

        #[test]
        fn gets_autocomplete_for_primitive_components() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Gr",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![CompletionItem {
                    label: "Grid".to_string(),
                    kind: CompletionItemKind::Class,
                    insert_text: "id".to_string(),
                    trigger_completion: false,
                },]
            );
        }
    }

    mod completion_component_attributes {
        use super::*;
        use crate::{
            InputContent, InputFile, RenderOptions, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
        };
        use std::path::PathBuf;

        #[test]
        fn gets_autocomplete_for_component_attributes() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Button ",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![
                    CompletionItem {
                        label: "href".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "href=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "download".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "download=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "download_as".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "download_as=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "target".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "target=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "variant".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "variant=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "size".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "size=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "width".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "width=\"".to_string(),
                        trigger_completion: true
                    }
                ]
            );
        }

        #[test]
        fn gets_autocomplete_for_component_attributes_prefix() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Button d",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![
                    CompletionItem {
                        label: "download".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "ownload=\"".to_string(),
                        trigger_completion: true
                    },
                    CompletionItem {
                        label: "download_as".to_string(),
                        kind: CompletionItemKind::Field,
                        insert_text: "ownload_as=\"".to_string(),
                        trigger_completion: true
                    },
                ]
            );
        }

        #[test]
        fn gets_autocomplete_for_custom_component_attributes() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
                },
                InputFile {
                    path: PathBuf::from("_components/example.md"),
                    content: InputContent::Text(
                        indoc! {r#"
                    ---
                    attributes:
                      - title: title
                        required: true
                    ---

                    the title is {@title}
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Component.Example ",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![CompletionItem {
                    label: "title".to_string(),
                    kind: CompletionItemKind::Field,
                    insert_text: "title=\"".to_string(),
                    trigger_completion: true
                },]
            );
        }

        #[test]
        fn gets_autocomplete_for_primitive_component_attributes() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Grid c",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![CompletionItem {
                    label: "cols".to_string(),
                    kind: CompletionItemKind::Field,
                    insert_text: "ols=\"".to_string(),
                    trigger_completion: true
                },]
            );
        }
    }

    mod completion_component_attributes_values {
        use super::*;
        use crate::{
            InputContent, InputFile, RenderOptions, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
        };
        use std::path::PathBuf;

        #[test]
        fn gets_autocomplete_for_component_attributes_values() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Card pad=\"",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![
                    CompletionItem {
                        label: "0".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "0\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "1".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "1\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "2".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "2\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "3".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "3\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "4".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "4\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "5".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "5\"".to_string(),
                        trigger_completion: false
                    },
                ]
            );
        }

        #[test]
        fn gets_autocomplete_for_component_attributes_values_insert_text() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Icon set=\"de",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![CompletionItem {
                    label: "devicon".to_string(),
                    kind: CompletionItemKind::Enum,
                    insert_text: "vicon\"".to_string(),
                    trigger_completion: false
                },]
            );
        }

        #[test]
        fn gets_autocomplete_for_primitive_component_attributes() {
            let files = vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::new()),
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
            let render_opts = RenderOptions::default();

            let completion_items = project.autocomplete(
                "The component is <Grid cols=\"",
                Path::new("README.md"),
                Some(&render_opts),
            );

            assert_eq!(
                completion_items,
                vec![
                    CompletionItem {
                        label: "1".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "1\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "2".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "2\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "3".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "3\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "4".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "4\"".to_string(),
                        trigger_completion: false
                    },
                    CompletionItem {
                        label: "5".to_string(),
                        kind: CompletionItemKind::Enum,
                        insert_text: "5\"".to_string(),
                        trigger_completion: false
                    }
                ]
            );
        }
    }
}
