use crate::icon::{Icon, IconDescription};
use crate::parser::is_external_link;
use crate::SETTINGS_FILE_NAME;
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::PathBuf;
use uriparse::{URIError, URI};

fn normalize_tab_path(path: &str) -> String {
    if is_external_link(path) || path == "/" {
        path.to_string()
    } else {
        format!("/{}", path.trim_start_matches('/').trim_end_matches('/'))
    }
}

#[derive(PartialEq, Clone, Debug, Serialize)]
pub struct StructureV2 {
    pub tabs: Vec<TabV2>,
}

pub fn parse_structure(input: &str) -> Result<StructureV2Description> {
    serde_yaml::from_str(input).map_err(|e| {
        Error::from_serde_yaml(
            e,
            Error::INVALID_STRUCTURE,
            "Invalid tabs".to_owned(),
            Some(PathBuf::from(SETTINGS_FILE_NAME)),
        )
    })
}

impl StructureV2 {
    pub fn from_tab_descriptions(tabs: Vec<TabDescription>) -> Self {
        let tabs = tabs.into_iter().map(|i| i.into()).collect::<Vec<TabV2>>();
        StructureV2 { tabs }
    }

    pub fn nav_paths(&self) -> Vec<String> {
        let mut tab_navs: Vec<_> = self
            .tabs
            .iter()
            .filter(|tab| !tab.is_external)
            .map(|tab| tab.href.clone())
            .collect();

        let subtab_navs: Vec<_> = self
            .subtabs()
            .iter()
            .filter(|subtab| !subtab.is_external)
            .map(|subtab| subtab.href.clone())
            .collect();

        tab_navs.extend(subtab_navs);

        tab_navs
    }

    pub fn build(input: &str) -> Result<Self> {
        let description = parse_structure(input)?;
        let mut tabs: Vec<TabV2> = Vec::new();

        for desc in description.tabs {
            tabs.push(desc.into());
        }

        Ok(StructureV2 { tabs })
    }

    pub fn subtabs(&self) -> Vec<TabV2> {
        self.tabs
            .iter()
            .flat_map(|t| t.subtabs.clone())
            .collect::<Vec<_>>()
    }

    pub fn verify(&self) -> Vec<Error> {
        let mut errors = vec![];

        // Ensure that there is at least one tab with path "/"
        if !self.tabs.iter().any(|tab| tab.href == "/") {
            errors.push(Error {
                code: Error::INVALID_STRUCTURE,
                message: String::from("One of the tabs must be root."),
                description: String::from("Expected a tab to have path \"/\". Found none."),
                file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                position: None,
            });
        }

        let mut seen = HashSet::new();

        for tab in &self.tabs {
            errors.extend(tab.verify(None));

            if !seen.insert(&tab.href) {
                errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from("Tabs can not share paths."),
                        description: format!("Multiple tabs share the path \"{}\".\nEach tab must have a unique path prefix.", tab.href),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
            }

            let mut seen_for_tab = HashSet::new();

            for subtab in &tab.subtabs {
                errors.extend(subtab.verify(Some(tab)));

                if !seen_for_tab.insert(&subtab.href) {
                    errors.push(Error {
                            code: Error::INVALID_STRUCTURE,
                            message: String::from("Subtabs can not share paths."),
                            description: format!("Multiple subtabs share the path \"{}\".\nEach subtab must have a unique path prefix.", subtab.href),
                            file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                            position: None,
                        });
                }
            }
        }

        errors
    }
}

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TabV2 {
    pub label: String,
    pub subtabs: Vec<TabV2>,
    pub href: String,
    pub is_external: bool,
    pub icon: Option<Icon>,
    #[serde(skip_serializing)]
    pub raw_path: Option<String>,
}

impl TabV2 {
    fn verify(&self, parent: Option<&TabV2>) -> Vec<Error> {
        let mut errors = vec![];
        let id = if parent.is_some() { "Subtab" } else { "Tab" };

        if let Some(icon) = &self.icon {
            if !icon.is_valid() {
                errors.push(Error {
                    code: Error::INVALID_STRUCTURE,
                    message: format!("Unknown icon set in {}", id.to_lowercase()),
                    description: format!(
                        "{} \"{}\" has an unknown icon set: \"{}\". Expected one of [{}].",
                        id,
                        self.label,
                        icon.set(),
                        Icon::available_sets()
                            .iter()
                            .map(|i| format!("\"{}\"", i))
                            .collect::<Vec<_>>()
                            .join(", "),
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                })
            }

            if self.is_external {
                errors.push(Error {
                    code: Error::INVALID_STRUCTURE,
                    message: format!("{} with external links cannot have an icon", id),
                    description: format!(
                        "{} \"{}\" has both an external link and an icon.",
                        id, self.label
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                })
            }
        }

        if self.is_external {
            errors.extend(self.verify_external(parent));
        } else {
            errors.extend(self.verify_internal(parent));
        }

        errors
    }

    fn verify_internal(&self, parent: Option<&TabV2>) -> Vec<Error> {
        let mut errors = vec![];
        let id = if parent.is_some() { "Subtab" } else { "Tab" };

        if let Some(raw_path) = &self.raw_path {
            if raw_path.contains("/./")
                || raw_path.contains("/../")
                || raw_path.ends_with("/.")
                || raw_path.ends_with("/..")
            {
                errors.push(Error {
                    code: Error::INVALID_STRUCTURE,
                    message: format!(
                        "Paths for {}s can't have relative paths.",
                        id.to_lowercase()
                    ),
                    description: format!(
                        "The path \"{}\" is not valid, since it contains a relative part.",
                        raw_path
                    ),
                    file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                    position: None,
                });
            }

            if let Some(parent) = parent {
                // This is a subtab
                let tab_prefix = format!("/{}", parent.href.split('/').nth(1).unwrap_or(""));

                if !raw_path.starts_with(&tab_prefix) {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: String::from(
                            "Paths within a tab must start with the parent's path.",
                        ),
                        description: format!(
                            "Expected path to start with \"{}\".\nFound \"{}\".",
                            tab_prefix, raw_path
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }
            }
        } else {
            errors.push(Error {
                code: Error::INVALID_STRUCTURE,
                message: format!("Missing \"path\" or \"external\" from {}.", id.to_lowercase()),
                description: format!(
                    "{} \"{}\" is not valid, since it is missing a path or an external URL. Add \"path: /\" or similar to your {}.",
                    id,
                    &self.label,
                    id.to_lowercase()
                ),
                file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                position: None,
            });
        }

        errors
    }

    fn verify_external(&self, parent: Option<&TabV2>) -> Vec<Error> {
        let mut errors = vec![];
        let id = if parent.is_some() { "Subtab" } else { "Tab" };

        if !self.subtabs.is_empty() {
            errors.push(Error {
                code: Error::INVALID_STRUCTURE,
                message: String::from("Tabs cannot combine subtabs and external URL."),
                description: format!("Tab \"{}\" has both subtabs and external URL.", self.label),
                file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                position: None,
            });
        }

        if let Err(err) = URI::try_from(self.href.as_str()) {
            match err {
                URIError::MissingScheme => {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: format!(
                            "{} external URL must have a scheme (for example https).",
                            id
                        ),
                        description: format!(
                            "Expected {} \"{}\" external URL to have a scheme.\nFound \"{}\".",
                            id.to_lowercase(),
                            self.label,
                            &self.href.chars().take(8).collect::<String>()
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }
                URIError::NotURI => {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: format!(
                            "Malformed URL for {}.", id.to_lowercase()
                        ),
                        description: format!(
                            "Expected {} \"{}\" external URL to be a valid url (e.g. https://www.example.com).\nFound \"{}\".",
                            id.to_lowercase(),
                            self.label,
                            &self.href.chars().take(8).collect::<String>()
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                position: None,
                    });
                }
                _ => {
                    errors.push(Error {
                        code: Error::INVALID_STRUCTURE,
                        message: format!("{} external URL is invalid.", id),
                        description: format!(
                            "{} \"{}\" external URL is invalid: {}",
                            id, self.label, err
                        ),
                        file: Some(PathBuf::from(SETTINGS_FILE_NAME)),
                        position: None,
                    });
                }
            }
        }

        errors
    }

    #[allow(dead_code)]
    fn subtab_paths(&self) -> Vec<&str> {
        self.subtabs
            .iter()
            .map(|subtab| subtab.href.as_ref())
            .collect::<Vec<_>>()
    }

    #[allow(dead_code)]
    pub fn prefix(&mut self, prefix: &str) -> &Self {
        self.href = format!("{}{}", prefix, self.href);
        self.subtabs.iter_mut().for_each(|subtab| {
            subtab.href = format!("{}{}", prefix, subtab.href);
        });

        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StructureV2Description {
    pub tabs: Vec<TabDescription>,
}

impl From<StructureV2Description> for StructureV2 {
    fn from(desc: StructureV2Description) -> StructureV2 {
        let tabs = desc
            .tabs
            .into_iter()
            .map(|i| i.into())
            .collect::<Vec<TabV2>>();

        StructureV2 { tabs }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TabDescription {
    pub path: Option<String>,
    pub external: Option<String>,
    pub label: String,
    #[serde(default)]
    pub subtabs: Vec<SubTabDescription>,
    pub icon: Option<IconDescription>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubTabDescription {
    path: Option<String>,
    external: Option<String>,
    label: String,
    icon: Option<IconDescription>,
}

impl From<TabDescription> for TabV2 {
    fn from(desc: TabDescription) -> TabV2 {
        let TabDescription {
            path,
            external,
            label,
            subtabs,
            icon,
            ..
        } = desc;

        let icon = icon.map(|i| i.resolve());

        let is_external = external.is_some() && path.is_none();

        let href = match (&path, external) {
            (Some(path), _) => normalize_tab_path(path),
            (_, Some(external)) => external,
            _ => String::from("/"),
        };

        let subtabs = subtabs
            .into_iter()
            .map(|i| i.into())
            .collect::<Vec<TabV2>>();

        TabV2 {
            label,
            subtabs,
            href,
            icon,
            is_external,
            raw_path: path,
        }
    }
}

impl From<SubTabDescription> for TabDescription {
    fn from(desc: SubTabDescription) -> TabDescription {
        let SubTabDescription {
            external,
            label,
            path,
            icon,
        } = desc.clone();

        TabDescription {
            path,
            external,
            label,
            subtabs: vec![],
            icon,
        }
    }
}

impl From<SubTabDescription> for TabV2 {
    fn from(desc: SubTabDescription) -> TabV2 {
        let tab_desc: TabDescription = desc.into();

        tab_desc.into()
    }
}

#[cfg(test)]
mod test {
    use crate::{project::Project, InputContent, InputFile, Structure};

    use super::*;

    struct ProjectBuilder {
        inputs: Vec<InputFile>,
    }

    impl ProjectBuilder {
        fn with_structure(structure: &str) -> Self {
            ProjectBuilder {
                inputs: vec![
                    InputFile {
                        path: PathBuf::from("README.md"),
                        content: InputContent::Text(String::from("")),
                    },
                    InputFile {
                        path: PathBuf::from(crate::SETTINGS_FILE_NAME),
                        content: InputContent::Text(
                            formatdoc! {"
                            ---
                            version: 2
                            title: An Project

                            {structure}
                            ", structure = structure}
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
    fn parses_structure() {
        let structure = indoc! {r#"
        tabs:
          - label: "Guides"
            path: /guides
            subtabs:
              - label: "Getting started"
                path: "/guides/getting-started"
              - label: "Examples"
                path: "/guides/examples"
        "#};

        let structure = parse_structure(structure).unwrap();
        let tabs = structure.tabs;

        assert_eq!(tabs[0].label.as_str(), "Guides");
        assert_eq!(tabs[0].subtabs[0].label.as_str(), "Getting started");
        assert_eq!(tabs[0].subtabs[1].label.as_str(), "Examples");
    }

    #[test]
    fn normalizes_tab_href() {
        let structure = indoc! {r#"
        tabs:
          - label: Starting slashes
            path: ///guides
          - label: Ending slashes
            path: /guides2////
          - label: No slashes
            path: guides3
        "#};

        let structure = StructureV2::build(structure).unwrap();
        let tabs = structure.tabs;

        assert_eq!(tabs[0].href.as_str(), "/guides");
        assert_eq!(tabs[1].href.as_str(), "/guides2");
        assert_eq!(tabs[2].href.as_str(), "/guides3");
    }

    #[test]
    fn verifies_one_path_is_root() {
        let structure = indoc! {r#"
        tabs:
          - label: Tab1
            path: foobar
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors
                .iter()
                .any(|e| e.message == "One of the tabs must be root."),
            "No error for no tabs path being root"
        );
    }

    #[test]
    fn verifies_subtab_paths_are_same() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
          - label: Tab1
            path: /tab1
            subtabs:
              - label: Section1
                path: /tab1/1
              - label: Section2
                path: /tab1/section1
          - label: Tab2
            path: /tab2
            subtabs:
              - label: Section3
                path: /tab2/section3/
              - label: Section4
                path: /asd/section4/
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors.iter().any(|e| e.message
                == "Paths within a tab must start with the parent's path."
                && e.description
                    == "Expected path to start with \"/tab2\".\nFound \"/asd/section4/\"."),
            "No error for subtab path not starting with parent path"
        );
    }

    #[test]
    fn verifies_tab_paths_are_unique() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
            subtabs:
              - label: Default
                path: /
          - label: Tab1
            path: /tab1
            subtabs:
              - label: Section1
                path: /tab1/
              - label: Section2
                path: /tab1/section1/
          - label: Tab2
            path: /tab1
            subtabs:
              - label: Section3
                path: /tab2/section3/
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Tabs can not share paths." && e.description == "Multiple tabs share the path \"/tab1\".\nEach tab must have a unique path prefix."), "No error for non-unique subtab paths");
    }

    #[test]
    fn verifies_subtab_paths_are_unique() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
            subtabs:
              - label: Default
                path: /
          - label: Tab1
            path: /tab1
            subtabs:
              - label: Section1
                path: /tab1
              - label: Section2
                path: /tab1
          - label: Tab2
            path: /tab2
            subtabs:
              - label: Section3
                path: /tab2/section3/
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Subtabs can not share paths." && e.description == "Multiple subtabs share the path \"/tab1\".\nEach subtab must have a unique path prefix."), "No error for non-unique subtab paths");
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

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(
            errors
                .iter()
                .any(|e| e.message == "Tabs cannot combine subtabs and external URL."),
            "No error for non-unique subtab paths"
        );
    }

    #[test]
    fn verifies_tab_path_or_external_exists() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Missing \"path\" or \"external\" from tab." && e.description == "Tab \"Default\" is not valid, since it is missing a path or an external URL. Add \"path: /\" or similar to your tab."), "No error for missing path or external");
    }

    #[test]
    fn verifies_subtab_path_or_external_exists() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
            subtabs:
              - label: Default subtab
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Missing \"path\" or \"external\" from subtab." && e.description == "Subtab \"Default subtab\" is not valid, since it is missing a path or an external URL. Add \"path: /\" or similar to your subtab."), "No error for missing path or external");
    }

    #[test]
    fn verifies_tab_href() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            external: ""
        "#};

        let builder = ProjectBuilder::with_structure(structure);

        let project = builder.build().unwrap();

        let errors = project
            .verify(None, None)
            .expect_err("Expected verification to fail");

        assert!(errors
            .iter()
            .any(|e| e.message == "Malformed URL for tab." && e.description == "Expected tab \"Default\" external URL to be a valid url (e.g. https://www.example.com).\nFound \"\"."), "No error for malformed uri");
    }

    #[test]
    fn tabs_can_have_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
            icon:
              set: lucide
              name: box
          - label: Tab2
            icon:
              set: lucide
              name: contrast
            path: /tab2
        "#};

        let mut builder = ProjectBuilder::with_structure(structure);
        builder.with_file("tab2/navigation.yaml", "");
        builder.with_file("tab2/README.md", "");

        let project = builder.build().unwrap();

        let result = project.verify(None, None);
        assert!(result.is_ok(), "Failed to build project: {:#?}", result);

        match project.structure() {
            Some(Structure::StructureV2(StructureV2 { tabs })) => {
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
            Some(Structure::StructureV1 { .. }) => {
                panic!("Expected StructureV2.");
            }
            None => panic!("No structure found"),
        }
    }

    #[test]
    fn subtabs_can_have_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
            subtabs:
              - label: First
                path: /
                icon:
                  set: lucide
                  name: box
              - label: Second
                path: /tab2
                icon:
                  set: devicon
                  name: sqlite
        "#};

        let mut builder = ProjectBuilder::with_structure(structure);
        builder.with_file("tab2/navigation.yaml", "");
        builder.with_file("tab2/README.md", "");

        let project = builder.build().unwrap();

        let result = project.verify(None, None);
        assert!(result.is_ok(), "Failed to build project: {:#?}", result);

        match project.structure() {
            Some(Structure::StructureV2(StructureV2 { tabs })) => {
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
            Some(Structure::StructureV1 { .. }) => {
                panic!("Expected StructureV2.");
            }
            None => panic!("No structure found"),
        }
    }

    #[test]
    fn verifies_subtabs_and_tab_icons() {
        let structure = indoc! {r#"
        tabs:
          - label: Default
            path: /
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

        let builder = ProjectBuilder::with_structure(structure);

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
            path: /
            subtabs:
              - label: Second
                path: /
          - label: Other
            external: https://www.example.com
            icon:
              set: devicon
              name: sqlite
        "#};

        let builder = ProjectBuilder::with_structure(structure);

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
