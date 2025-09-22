use serde::Serialize;

use crate::{
    markdown,
    navigation::{Item, Section},
    render_context::RenderContext,
    Project, RenderOptions, Result,
};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Breadcrumb {
    Label { text: String },
    Link { href: String, label: String },
}

pub(crate) fn compute(
    uri_path: &str,
    project: &Project,
    opts: Option<&RenderOptions>,
) -> Result<Vec<Breadcrumb>> {
    let subtab_path = project
        .get_subtab_path_by_uri_path(uri_path)
        .unwrap_or("/".to_string());

    let mut ctx = RenderContext::default();
    ctx.with_maybe_options(opts);

    let navigation = project.navigation(opts, &subtab_path)?;

    // NOTE: The URL for the page we're requesting _won't_ have the prefix for e.g. a preview build
    // or a version, but the outputted links _should_ have the prefix. So for any comparisons with
    // the navigation we need to compare against the prefix'd uri path.
    let uri_path = markdown::parser::to_final_link(uri_path, &ctx);

    let mut out = vec![];

    for section in &navigation.sections {
        if section.has_link_to(&uri_path) {
            walk_section(&uri_path, section, &mut out);
            break;
        }
    }

    Ok(out)
}

fn walk_section(uri_path: &str, section: &Section, out: &mut Vec<Breadcrumb>) {
    if let Some(heading) = &section.heading {
        out.push(Breadcrumb::Label {
            text: heading.clone(),
        });
    }

    for item in &section.items {
        if item.has_link_to(uri_path) {
            walk_item(uri_path, item, out);
            break;
        }
    }
}

fn walk_item(uri_path: &str, item: &Item, out: &mut Vec<Breadcrumb>) {
    if item.matches_href(uri_path) {
        return;
    }

    if item.has_link_to(uri_path) {
        if item.is_subheading() {
            out.push(Breadcrumb::Label {
                text: item.label().to_owned(),
            });
        } else if item.href().is_some() {
            out.push(Breadcrumb::Link {
                href: item.href().unwrap().to_owned(),
                label: item.label().to_owned(),
            });
        }

        if let Some(children) = item.items() {
            for child in children {
                walk_item(uri_path, child, out);
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use crate::{InputContent, InputFile, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME};

    use super::*;

    #[test]
    fn gathers_breadcrumbs_section_heading() {
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
                path: PathBuf::from("fizz/bar.md"),
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
                  items:
                    - label: Fizz
                      href: fizz/bar.md
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

        let project = Project::from_file_list(files.clone()).unwrap();
        let page = project.get_page_by_uri_path("/fizz/bar").unwrap();

        assert_eq!(
            page.breadcrumbs(None),
            vec![Breadcrumb::Label {
                text: "Something".to_string()
            }]
        )
    }

    #[test]
    fn gathers_breadcrumbs_section_nested_links() {
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
                path: PathBuf::from("fizz/bar.md"),
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
                - heading: Some
                  items:
                    - label: Parent
                      href: /
                      items:
                        - label: Fizz
                          href: fizz/bar.md
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

        let project = Project::from_file_list(files.clone()).unwrap();
        let page = project.get_page_by_uri_path("/fizz/bar").unwrap();

        assert_eq!(
            page.breadcrumbs(None),
            vec![
                Breadcrumb::Label {
                    text: "Some".to_string()
                },
                Breadcrumb::Link {
                    href: "/".to_string(),
                    label: "Parent".to_string()
                }
            ]
        )
    }

    #[test]
    fn gathers_breadcrumbs_section_subheadings() {
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
                path: PathBuf::from("fizz/bar.md"),
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
                  items:
                    - subheading: Else
                      items:
                        - label: Fizz
                          href: fizz/bar.md
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

        let project = Project::from_file_list(files.clone()).unwrap();
        let page = project.get_page_by_uri_path("/fizz/bar").unwrap();

        assert_eq!(
            page.breadcrumbs(None),
            vec![
                Breadcrumb::Label {
                    text: "Something".to_string()
                },
                Breadcrumb::Label {
                    text: "Else".to_string()
                }
            ]
        )
    }

    #[test]
    fn gathers_breadcrumbs_only_first_match() {
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
                path: PathBuf::from("fizz/bar.md"),
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
                  items:
                    - label: Fizz
                      href: fizz/bar.md

                - heading: Duplicate that should be ignored
                  items:
                    - label: Fizz
                      href: fizz/bar.md
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

        let project = Project::from_file_list(files.clone()).unwrap();
        let page = project.get_page_by_uri_path("/fizz/bar").unwrap();

        assert_eq!(
            page.breadcrumbs(None),
            vec![Breadcrumb::Label {
                text: "Something".to_string()
            },]
        )
    }

    #[test]
    fn gathers_breadcrumbs_with_prefixed_links() {
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
                path: PathBuf::from("fizz/bar.md"),
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
                - heading: Some
                  items:
                    - label: Parent
                      href: /
                      items:
                        - label: Fizz
                          href: fizz/bar.md
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

        let project = Project::from_file_list(files.clone()).unwrap();
        let page = project.get_page_by_uri_path("/fizz/bar").unwrap();

        let opts = RenderOptions {
            prefix_link_urls: Some("/prefix".to_string()),
            ..Default::default()
        };

        assert_eq!(
            page.breadcrumbs(Some(&opts)),
            vec![
                Breadcrumb::Label {
                    text: "Some".to_string()
                },
                Breadcrumb::Link {
                    href: "/prefix/".to_string(),
                    label: "Parent".to_string()
                }
            ]
        )
    }

    #[test]
    fn ignores_previous_siblings() {
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
                path: PathBuf::from("fizz/bar.md"),
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
                - heading: Some
                  items:
                    - label: Parent
                      href: /
                      items:
                        - label: Ignore me
                          href: /
                        - label: Fizz
                          href: fizz/bar.md
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

        let project = Project::from_file_list(files.clone()).unwrap();
        let page = project.get_page_by_uri_path("/fizz/bar").unwrap();

        assert_eq!(
            page.breadcrumbs(None),
            vec![
                Breadcrumb::Label {
                    text: "Some".to_string()
                },
                Breadcrumb::Link {
                    href: "/".to_string(),
                    label: "Parent".to_string()
                }
            ]
        )
    }
}
