use std::collections::HashSet;
use std::path::PathBuf;

use crate::icon::{Icon, IconDescription};
use crate::structure_v2::{StructureV2, TabV2};
use crate::STRUCTURE_FILE_NAME;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};

#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(tag = "version")]
pub enum Structure {
    StructureV1(StructureV1),
    StructureV2(StructureV2),
}

#[derive(PartialEq, Clone, Debug, Serialize)]
#[serde(tag = "version")]
pub enum Tab {
    #[serde(rename = "1")]
    TabV1(TabV1),
    #[serde(rename = "2")]
    TabV2(TabV2),
}

impl Tab {
    pub fn get_v2(&self) -> Option<&TabV2> {
        match self {
            Tab::TabV1(_) => None,
            Tab::TabV2(tab) => Some(tab),
        }
    }
}

#[derive(PartialEq, Clone, Debug, Serialize)]

pub struct StructureV1 {
    pub tabs: Vec<TabV1>,
}

impl StructureV1 {
    pub fn subtabs(&self) -> Vec<SubTab> {
        self.tabs
            .iter()
            .flat_map(|t| t.subtabs.clone())
            .collect::<Vec<_>>()
    }

    pub fn nav_paths(&self) -> Vec<String> {
        self.subtabs()
            .iter()
            .map(|subtab| subtab.path.clone())
            .collect()
    }

    pub fn verify(&self) -> Vec<Error> {
        let mut errors = vec![];

        let mut seen = HashSet::new();

        if let Some(tab) = self.tabs.first() {
            if let Some(prefix) = tab.prefix() {
                if prefix != "/" {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("First subtab path must be root."),
                        description: format!(
                            "Expected first path to be \"/\".\nFound \"{}\".",
                            prefix
                        ),
                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                        position: None,
                    });
                }

                for tab1_subtab in tab.subtab_paths() {
                    for other_tab in self.tabs.iter().skip(1) {
                        for other_subtab in other_tab.subtab_paths() {
                            if tab1_subtab == other_subtab {
                                errors.push(Error {
                                        code: Error::INVALID_STRUCTURE,
                                        message: String::from("Tabs can not share paths."),
                                        description: format!("Multiple tabs share the path \"{}\".\nEach tab must have a unique path prefix.", tab1_subtab),
                                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
            position: None,
                                    });
                            }
                        }
                    }
                }
            }
        }

        for tab in &self.tabs {
            errors.extend(tab.verify());

            if let Some(prefix) = tab.prefix() {
                if !seen.insert(prefix.clone()) {
                    errors.push(Error {
                            code: Error::INVALID_STRUCTURE,
                            message: String::from("Tabs can not share paths."),
                            description: format!("Multiple tabs share the path \"{}\".\nEach tab must have a unique path prefix.", prefix),
                            file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
            position: None,
                        });
                }
            }
        }
        errors
    }
}

impl Structure {
    pub fn builder() -> StructureBuilder {
        StructureBuilder::default()
    }

    pub fn tabs(&self) -> Vec<Tab> {
        match self {
            Structure::StructureV1(v1) => v1.tabs.iter().map(|t| Tab::TabV1(t.clone())).collect(),
            Structure::StructureV2(v2) => v2.tabs.iter().map(|t| Tab::TabV2(t.clone())).collect(),
        }
    }
}

/// Build the tab structure.
pub(crate) fn build(input: &str) -> Result<Structure> {
    let description = parse_structure(input)?;
    let mut tabs = Vec::new();

    for desc in description.tabs {
        if let Some(tab) = desc.resolve() {
            tabs.push(tab);
        }
    }

    Ok(Structure::StructureV1(StructureV1 { tabs }))
}

/// Verify that the structure is properly formed
pub(crate) fn verify(structure: &Structure) -> Vec<Error> {
    match structure {
        Structure::StructureV1(v1) => v1.verify(),
        Structure::StructureV2(v2) => v2.verify(),
    }
}

pub fn parse_structure(input: &str) -> Result<StructureDescription> {
    serde_yaml::from_str(input).map_err(|e| {
        Error::from_serde_yaml(
            e,
            Error::INVALID_STRUCTURE,
            "Invalid structure.yaml".to_owned(),
            Some(PathBuf::from("structure.yaml")),
        )
    })
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TabV1 {
    pub label: String,
    pub subtabs: Vec<SubTab>,
    pub href: String,
    pub external_href: Option<String>,
    pub icon: Option<Icon>,
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct SubTab {
    pub label: String,
    pub path: String,
    pub href: String,
    pub icon: Option<Icon>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureDescription {
    pub tabs: Vec<TabDescription>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabDescription {
    pub label: String,
    pub subtabs: Option<Vec<SubTabDescription>>,
    pub external: Option<String>,
    pub icon: Option<IconDescription>,
}

impl TabV1 {
    fn verify(&self) -> Vec<Error> {
        let mut errors = vec![];

        if let Some(icon) = &self.icon {
            if !icon.is_valid() {
                errors.push(Error {
                    code: Error::INVALID_STRUCTURE,
                    message: String::from("Unknown icon set in tab"),
                    description: format!(
                        "Tab \"{}\" has an unknown icon set: \"{}\". Expected one of [{}].",
                        self.label,
                        icon.set(),
                        Icon::available_sets()
                            .iter()
                            .map(|i| format!("\"{}\"", i))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                    position: None,
                })
            }
        }

        if self.external_href.is_some() {
            self.verify_href(&mut errors);

            if self.icon.is_some() {
                errors.push(Error {
                    code: Error::INVALID_STRUCTURE,
                    message: "Tab with external links cannot have an icon".to_string(),
                    description: format!(
                        "Tab \"{}\" has both an external link and an icon.",
                        self.label
                    ),
                    file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                    position: None,
                })
            }
        } else {
            self.verify_subtabs(&mut errors);
        }

        errors
    }

    fn verify_href(&self, errors: &mut Vec<Error>) {
        if !self.subtabs.is_empty() {
            errors.push(Error {
                code: Error::INVALID_STRUCTURE,
                message: String::from("Tabs cannot combine subtabs and external URL."),
                description: format!("Tab \"{}\" has both subtabs and external URL.", self.label),
                file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                position: None,
            });
        }

        if let Some(href) = &self.external_href {
            if !href.starts_with("http://") && !href.starts_with("https://") {
                errors.push(Error {
                    code: Error::INVALID_STRUCTURE,
                    message: String::from("Tab external URL must start with https://."),
                    description: format!(
                        "Expected tab \"{}\" external URL to start with https://.\nFound \"{}\".",
                        self.label,
                        &href.chars().take(8).collect::<String>()
                    ),
                    file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                    position: None,
                });
            }
        }
    }

    fn verify_subtabs(&self, errors: &mut Vec<Error>) {
        if self.subtabs.is_empty() {
            errors.push(Error {
                code: Error::INVALID_STRUCTURE,
                message: String::from("Tabs must have at least one subtab."),
                description: format!("Tab \"{}\" has no subtabs.", self.label),
                file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                position: None,
            });

            return;
        }

        if let Some(subtab_path) = self.prefix() {
            let paths = self.subtab_paths();

            paths.iter().for_each(|s| {
                if !s.starts_with('/') {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("Paths for subtabs must start with a slash."),
                        description: format!("Expected path to start with \"/\".\nFound {}.", s),
                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                        position: None,
                    });
                } else if !s.ends_with('/') {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("Paths for subtabs must end in a slash."),
                        description: format!("Expected path to end with \"/\".\nFound \"{}\".", s),
                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                        position: None,
                    });
                } else if s.contains('.') {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("Paths for subtabs cannot contain dots."),
                        description: format!(
                            "The path \"{}\" is not valid, since it contains dots.",
                            s
                        ),
                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                        position: None,
                    });
                } else if !s.starts_with(&subtab_path) {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("Paths within a tab must start with the same path."),
                        description: format!(
                            "Expected path to start with \"{}\".\nFound \"{}\".",
                            subtab_path, s
                        ),
                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                        position: None,
                    });
                }
            });
        } else {
            errors.push(Error {
                code: Error::INVALID_STRUCTURE,
                message: String::from("Paths must be proper."),
                description: format!("Failed to parse the path for tab \"{}\".", self.label),
                file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                position: None,
            });
        }

        for subtab in &self.subtabs {
            if let Some(icon) = &subtab.icon {
                if !icon.is_valid() {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("Unknown icon set in subtab"),
                        description: format!(
                            "Subtab \"{}\" has an unknown icon set: \"{}\". Expected one of [{}].",
                            subtab.label,
                            icon.set(),
                            Icon::available_sets()
                                .iter()
                                .map(|i| format!("\"{}\"", i))
                                .collect::<Vec<_>>()
                                .join(", "),
                        ),
                        file: Some(PathBuf::from(STRUCTURE_FILE_NAME)),
                        position: None,
                    })
                }
            }
        }
    }

    pub fn subtab_path(&self) -> Option<&str> {
        self.subtab_paths().first().copied()
    }

    pub fn subtab_paths(&self) -> Vec<&str> {
        self.subtabs
            .iter()
            .map(|subtab| subtab.path.as_ref())
            .collect::<Vec<_>>()
    }

    // Returns the first part of the tab path, normalized.
    pub fn prefix(&self) -> Option<String> {
        match self.subtab_path() {
            Some("/") => Some("/".to_string()),
            Some(p) => p
                .trim_start_matches('/')
                .trim_end_matches('/')
                .split('/')
                .next()
                .map(|path| format!("/{}/", path)),
            None => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubTabDescription {
    label: String,
    path: String,
    icon: Option<IconDescription>,
}

impl TabDescription {
    fn resolve(self) -> Option<TabV1> {
        let TabDescription {
            label,
            subtabs,
            external: external_href,
            icon,
        } = self;

        let subtabs = subtabs
            .map(|i| {
                i.into_iter()
                    .filter_map(|i| i.resolve())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let (href, external_href) = match external_href {
            Some(href) => (href.clone(), Some(href)),
            None => (
                subtabs
                    .first()
                    .map(|i| i.href.to_owned())
                    .unwrap_or("/".to_string()),
                None,
            ),
        };

        let icon = icon.map(|i| i.resolve());

        let tab = TabV1 {
            label,
            subtabs,
            href,
            external_href,
            icon,
        };
        Some(tab)
    }
}

impl SubTabDescription {
    fn resolve(self) -> Option<SubTab> {
        let SubTabDescription { label, path, icon } = self;
        let href = format!("/{}", path.trim_start_matches('/').trim_end_matches('/'));

        let icon = icon.map(|i| i.resolve());

        Some(SubTab {
            label,
            path,
            href,
            icon,
        })
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct StructureLocal {
    pub tabs: Vec<TabV1>,
    pub active_tab: Option<TabV1>,
    pub active_subtab: Option<SubTab>,
}

#[derive(Default)]
pub struct StructureBuilder {
    tabs: Vec<TabV1>,
    prefix_link_urls: Option<String>,
    subtab_path: Option<String>,
}

impl StructureBuilder {
    pub fn new() -> StructureBuilder {
        StructureBuilder {
            tabs: Vec::new(),
            prefix_link_urls: None,
            subtab_path: None,
        }
    }
    pub fn with_tabs(mut self, tabs: Vec<TabV1>) -> StructureBuilder {
        self.tabs = tabs;
        self
    }

    pub fn with_prefix(mut self, prefix_link_urls: Option<&str>) -> StructureBuilder {
        self.prefix_link_urls = prefix_link_urls.map(String::from);
        self
    }

    pub fn with_active(mut self, subtab_path: Option<&str>) -> StructureBuilder {
        self.subtab_path = subtab_path.map(String::from);
        self
    }

    pub fn build(self) -> StructureLocal {
        let prefix = self.prefix_link_urls.as_ref();
        let mut active_subtab: Option<SubTab> = None;
        let mut active_tab: Option<TabV1> = None;

        let tabs: Vec<TabV1> = self
            .tabs
            .into_iter()
            .map(|t| TabV1 {
                subtabs: t
                    .subtabs
                    .into_iter()
                    .map(|i| SubTab {
                        href: Self::prefix_link(i.href, prefix),
                        label: i.label,
                        path: i.path,
                        icon: i.icon,
                    })
                    .collect(),
                label: t.label,
                href: Self::prefix_link(t.href, prefix),
                external_href: t.external_href,
                icon: t.icon,
            })
            .collect();

        if let Some(subtab_path) = self.subtab_path {
            'outer: for tab in tabs.iter() {
                for subtab in tab.subtabs.iter() {
                    if subtab.path == subtab_path {
                        active_subtab = Some(subtab.clone());
                        active_tab = Some(tab.clone());
                        break 'outer;
                    }
                }
            }
        }

        StructureLocal {
            tabs,
            active_subtab,
            active_tab,
        }
    }

    fn prefix_link(url: String, prefix: Option<&String>) -> String {
        match prefix {
            Some(prefix) => {
                let mut rewrite = String::from(prefix.strip_suffix('/').unwrap_or(prefix));
                rewrite.push('/');
                rewrite.push_str(url.strip_prefix('/').unwrap_or(&url));
                rewrite
            }
            None => url,
        }
    }
}

#[cfg(test)]
mod test {
    use crate::{project::Project, InputContent, InputFile};

    use super::*;

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
                        content: InputContent::Text("---\n- heading: Asd".to_owned()),
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
        let structure = indoc! {r#"
        tabs:
          - label: "Guides"
            subtabs:
              - label: "Getting started"
                path: "/guides/getting-started"
              - label: "Examples"
                path: "/guides/examples"
        "#};

        let structure = parse_structure(structure).unwrap();
        let tabs = structure.tabs;

        assert_eq!(tabs[0].label.as_str(), "Guides");
        assert_eq!(
            tabs[0].subtabs.as_ref().unwrap()[0].label.as_str(),
            "Getting started"
        );
        assert_eq!(
            tabs[0].subtabs.as_ref().unwrap()[1].label.as_str(),
            "Examples"
        );
    }

    #[test]
    fn tab_prefix_returns_canonical_tab_path() {
        let structure = indoc! {r#"
        tabs:
          - label: Tab1
            subtabs:
              - label: Section1
                path: /tab1
          - label: Tab2
            subtabs:
              - label: Section2
                path: tab2/section2/
          - label: Tab3
            subtabs:
              - label: Section3
                path: tab3/section3
              - label: Section4
                path: tab3
        "#};

        let structure = build(structure).unwrap();

        match structure {
            Structure::StructureV1(StructureV1 { tabs }) => {
                assert_eq!(tabs[0].prefix(), Some("/tab1/".to_string()));
                assert_eq!(tabs[1].prefix(), Some("/tab2/".to_string()));
                assert_eq!(tabs[2].prefix(), Some("/tab3/".to_string()));
                assert_eq!(tabs[2].prefix(), Some("/tab3/".to_string()));
            }
            _ => panic!("Expected StructureV1"),
        }
    }

    #[test]
    fn verifies_tab_paths_start_and_end_with_slash() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: Default
                path: /
          - label: Tab1
            subtabs:
              - label: Section1
                path: /tab1
          - label: Tab2
            subtabs:
              - label: Section2
                path: tab2/asd/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors
                .iter()
                .any(|e| e.message == "Paths for subtabs must end in a slash."),
            "No error for paths not ending in slashes"
        );

        assert!(
            errors
                .iter()
                .any(|e| e.message == "Paths for subtabs must start with a slash."),
            "No error for paths not starting with slashes"
        );
    }

    #[test]
    fn verifies_first_subtab_path_is_root() {
        let structure = indoc! {r#"
        tabs:
          - label: Tab1
            subtabs:
              - label: Section1
                path: /asd/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors
                .iter()
                .any(|e| e.message == "First subtab path must be root."),
            "No error for first subtab path not being root"
        );
    }

    #[test]
    fn verifies_subtab_paths_are_same() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: Default
                path: /
          - label: Tab1
            subtabs:
              - label: Section1
                path: /tab1/
              - label: Section2
                path: /tab1/section1/
          - label: Tab2
            subtabs:
              - label: Section3
                path: /tab2/section3/
              - label: Section4
                path: /asd/section4/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors.iter().any(
                |e| e.message == "Paths within a tab must start with the same path."
                    && e.description
                        == "Expected path to start with \"/tab2/\".\nFound \"/asd/section4/\"."
            ),
            "No error for subtab path not starting with parent path"
        );
    }

    #[test]
    fn verifies_tab_subtab_paths_are_unique() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: Default
                path: /
          - label: Tab1
            subtabs:
              - label: Section1
                path: /tab1/
              - label: Section2
                path: /tab1/section1/
          - label: Tab2
            subtabs:
              - label: Section3
                path: /tab1/section3/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Tabs can not share paths." && e.description == "Multiple tabs share the path \"/tab1/\".\nEach tab must have a unique path prefix."), "No error for non-unique subtab paths");
    }

    #[test]
    fn verifies_home_tab_subtab_paths_special_case() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: Default
                path: /
              - label: Jinxed
                path: /tab1/
          - label: Tab1
            subtabs:
              - label: Sub-tab 1
                path: /tab1/
              - label: Sub-tab 2
                path: /tab1/section1/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Tabs can not share paths." && e.description == "Multiple tabs share the path \"/tab1/\".\nEach tab must have a unique path prefix."), "No error for non-unique subtab paths");
    }

    #[test]
    fn verifies_tab_paths_are_somewhat_valid() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: Default
                path: /
          - label: Tab1
            subtabs:
              - label: Section1
                path: /../tab1/
          - label: Tab2
            subtabs:
              - label: Section3
                path: /tab2/../section3/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors
                .iter()
                .any(|e| e.message == "Paths for subtabs cannot contain dots."
                    && e.description
                        == "The path \"/../tab1/\" is not valid, since it contains dots."),
            "No error for paths containing dots"
        );

        assert!(
            errors
                .iter()
                .filter(|e| e.message == "Paths for subtabs cannot contain dots.")
                .count()
                == 2,
            "No error for paths containing dots"
        );
    }

    #[test]
    fn verifies_tab_subtabs_and_href_mutually_exclusive() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            external: https://example.com
            subtabs:
              - label: Default
                path: /
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert_eq!(
            &errors[0].message,
            "Tabs cannot combine subtabs and external URL."
        );
    }

    #[test]
    fn verifies_tab_href() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            external: ""
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        let error = errors
            .iter()
            .find(|e| e.message == "Tab external URL must start with https://.")
            .unwrap();

        assert_eq!(&error.message, "Tab external URL must start with https://.");
        assert_eq!(
            &error.description,
            "Expected tab \"Default\" external URL to start with https://.\nFound \"\"."
        );
    }

    #[test]
    fn tabs_can_have_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            icon:
              set: lucide
              name: box
            subtabs:
              - label: Default
                path: /
          - label: Tab2
            icon:
              set: lucide
              name: contrast
            subtabs:
              - label: Section2
                path: /tab2/
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);
        builder.with_file("tab2/navigation.yaml", "");
        builder.with_file("tab2/README.md", "");

        let project = builder.build().unwrap();

        let result = project.verify(None, None);
        assert!(result.is_ok(), "Failed to build project: {:#?}", result);

        match project.structure() {
            Some(Structure::StructureV1(StructureV1 { tabs })) => {
                assert_eq!(
                    tabs.first().as_ref().unwrap().icon.as_ref().unwrap().name(),
                    "box".to_owned()
                );
                assert_eq!(
                    tabs.first().as_ref().unwrap().icon.as_ref().unwrap().set(),
                    "lucide".to_owned()
                );
                assert_eq!(
                    tabs.get(1).as_ref().unwrap().icon.as_ref().unwrap().name(),
                    "contrast"
                );
                assert_eq!(
                    tabs.get(1).as_ref().unwrap().icon.as_ref().unwrap().set(),
                    "lucide"
                );
            }
            Some(Structure::StructureV2 { .. }) => {
                panic!("Expected StructureV1.");
            }
            None => panic!("No structure found"),
        }
    }

    #[test]
    fn subtabs_can_have_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: First
                path: /
                icon:
                  set: lucide
                  name: box
              - label: Second
                path: /
                icon:
                  set: devicon
                  name: sqlite
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);
        builder.with_file("tab2/navigation.yaml", "");

        let project = builder.build().unwrap();

        let result = project.verify(None, None);
        assert!(result.is_ok(), "Failed to build project: {:#?}", result);

        match project.structure() {
            Some(Structure::StructureV1(StructureV1 { tabs })) => {
                let tab = tabs.first().unwrap().clone();

                assert_eq!(
                    tab.subtabs.first().unwrap().icon.as_ref().unwrap().name(),
                    "box"
                );
                assert_eq!(
                    tab.subtabs.first().unwrap().icon.as_ref().unwrap().set(),
                    "lucide"
                );

                assert_eq!(
                    tab.subtabs.get(1).unwrap().icon.as_ref().unwrap().name(),
                    "sqlite"
                );
                assert_eq!(
                    tab.subtabs.get(1).unwrap().icon.as_ref().unwrap().set(),
                    "devicon"
                );
            }
            Some(Structure::StructureV2 { .. }) => {
                panic!("Expected StructureV1.");
            }
            None => panic!("No structure found"),
        }
    }

    #[test]
    fn verifies_subtabs_and_tab_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            icon:
              set: nope
              name: box
            subtabs:
              - label: Second
                path: /
                icon:
                  set: nope
                  name: sqlite
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project.verify(None, None).unwrap_err();

        let tab_error = errors
            .iter()
            .find(|e| e.message == "Unknown icon set in tab")
            .unwrap();

        let subtab_error = errors
            .iter()
            .find(|e| e.message == "Unknown icon set in subtab")
            .unwrap();

        assert_eq!(tab_error.message, "Unknown icon set in tab");
        assert_eq!(
            tab_error.description,
            "Tab \"Default\" has an unknown icon set: \"nope\". Expected one of [\"lucide\", \"devicon\"].",
        );
        assert_eq!(subtab_error.message, "Unknown icon set in subtab");
        assert_eq!(
            subtab_error.description,
            "Subtab \"Second\" has an unknown icon set: \"nope\". Expected one of [\"lucide\", \"devicon\"].",
        );
    }

    #[test]
    fn verifies_external_tab_links_do_not_support_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            subtabs:
              - label: Second
                path: /
          - label: Other
            external: https://www.example.com
            icon:
              set: devicon
              name: sqlite
        "#};

        let mut builder = ProjectBuilder::default();
        builder.with_file(crate::STRUCTURE_FILE_NAME, structure);

        let project = builder.build().unwrap();

        let errors = project.verify(None, None).unwrap_err();

        assert_eq!(
            errors[0].message,
            "Tab with external links cannot have an icon"
        );
        assert_eq!(
            errors[0].description,
            "Tab \"Other\" has both an external link and an icon."
        );
    }
}
