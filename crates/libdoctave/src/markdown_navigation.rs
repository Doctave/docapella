//! DEPRECATION NOTICE
//!
//! This is the original implementation of navigation via a _Navigation.md file
//! which is no longer supported.

use crate::markdown::{self, Node, NodeKind};
use crate::render_context::RenderContext;
use crate::{navigation, utils::empty_vector, Error, Result};

pub struct Navigation {}

impl Navigation {
    /// Processes a Markdown file into a list of navigation sections.
    ///
    /// This is some pretty gnarly code, because we are essentially traversing
    /// the AST looking for specific nodes, and we can bail at any moment.
    pub(crate) fn from_markdown(
        markdown_string: &str,
        ctx: &RenderContext,
    ) -> Result<navigation::Navigation> {
        let mut sections = Vec::new();
        let mut current_section = NavSection::default();

        for node in markdown::ast(markdown_string, ctx)?.children {
            match (&mut current_section, &node) {
                // Case 1: We hit a heading, and our current section does not have one yet
                //         Just update the heading, and move on. We'll either hit another
                //         heading, starting a new section, or a list with links in this
                //         section..
                (
                    NavSection { heading: None, .. },
                    Node {
                        kind: NodeKind::Heading { .. },
                        ..
                    },
                ) => {
                    current_section.heading = Some(String::new());

                    // Iterate the children of the heading for text
                    for node in &node.children {
                        let inner_text = node.inner_text();
                        if !inner_text.is_empty() {
                            // We've just set the heading above, so unwrap OK.
                            current_section.heading.as_mut().unwrap().push_str(&inner_text);
                        }
                    }
                }
                // Case 2: We hit a heading, but we already just parsed a heading.
                //         Create a new section, and assume we had a section without
                //         links, which we push.
                (
                    NavSection { heading: Some(_), .. },
                    Node {
                        kind: NodeKind::Heading { .. },
                        ..
                    },
                ) => {
                    sections.push(current_section.clone());
                    current_section = NavSection::default();
                    current_section.heading = Some(String::new());

                    // Iterate the children of the heading for text
                    for node in &node.children {
                        let inner_text = node.inner_text();
                        if !inner_text.is_empty() {
                            // We've just set the heading above, so unwrap OK.
                            current_section.heading.as_mut().unwrap().push_str(
                                &inner_text
                            );
                        }
                    }
                }
                // Case 3: We hit a list. Process the whole hierarchy.
                (
                    _,
                    Node {
                        kind: NodeKind::List { ordered: false, .. },
                        ..
                    },
                ) => {
                    let links = Self::process_list(&node)?;
                    current_section.items = links;

                    sections.push(current_section.clone());
                    current_section = NavSection::default();
                }
                other => return Err(error(format!("Invalid element found in Navigation: {:?}. Only headings and nested lists of links are allowd.", other)))
            }
        }

        // Ugh. This feels wrong.
        if current_section.heading.is_some() && sections.last() != Some(&current_section) {
            sections.push(current_section);
        }

        Ok(navigation::Navigation::new(
            sections.iter().map(|s| s.into()).collect::<Vec<_>>(),
        ))
    }

    fn process_list(list: &Node) -> Result<Vec<NavItem>> {
        let mut items = Vec::new();

        for node in &list.children {
            match &node.kind {
                NodeKind::ListItem { .. } => {
                    if node.children.is_empty() {
                        // Empty list item. Just skip it.
                        continue;
                    }

                    let mut link_url = String::new();
                    let mut link_text = String::new();

                    let mut children = node.children.iter();

                    // Step 1: Read what should be a Paragraph with a Link inside it.
                    let link_node = children
                        .next()
                        .and_then(|c| c.children.first())
                        .ok_or_else(|| error(String::new()))?;

                    match &link_node.kind {
                        NodeKind::Link { url, .. } => {
                            link_url.push_str(std::str::from_utf8(url.as_ref()).map_err(|_| {
                                error("Invalid UTF-8 sequence in navigation URL".to_owned())
                            })?);

                            // Parse the text for the link
                            for node in &link_node.children {
                                let inner_text = node.inner_text();
                                if !inner_text.is_empty() {
                                    link_text.push_str(&inner_text);
                                }
                            }
                        }
                        other => {
                            return Err(error(format!(
                                "Unexpected element in navigation. Expected a link, found {:?}",
                                other
                            )))
                        }
                    }

                    // Step 2: Check if we have children, and recurse if we do. Otherwise
                    //         set a blank list of children.
                    let item_children = if let Some(second_child) = children.next() {
                        match &second_child.kind {
                            NodeKind::List { ordered: false, .. } => {
                                Self::process_list(second_child)?
                            },
                            other => {
                                return Err(error(format!(
                                    "Unexpected child for navigation list item. Expected a nested list, found {:?}",
                                    other
                                )))
                            }
                        }
                    } else {
                        Vec::new()
                    };

                    // Step 3: Create the NavItem
                    items.push(NavItem {
                        text: link_text,
                        url: link_url,
                        children: item_children,
                    });
                }
                other => {
                    return Err(error(format!(
                        "Unexpected element in navigation. Expected list item, found {:?}",
                        other
                    )))
                }
            }
        }

        Ok(items)
    }
}

fn error(description: String) -> Error {
    Error {
        code: Error::NAVIGATION_ERROR,
        message: "Error parsing navigation".to_owned(),
        description,
        file: None,
        position: None,
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct NavSection {
    pub heading: Option<String>,
    pub items: Vec<NavItem>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NavItem {
    pub text: String,
    pub url: String,
    #[serde(skip_serializing_if = "empty_vector", default = "Vec::new")]
    pub children: Vec<NavItem>,
}

impl From<&NavSection> for navigation::Section {
    fn from(other: &NavSection) -> navigation::Section {
        navigation::Section {
            heading: other.heading.clone(),
            collapsed: false,
            collapsible: false,
            items: other.items.iter().map(|i| i.into()).collect::<Vec<_>>(),
        }
    }
}

impl From<&NavItem> for navigation::Item {
    fn from(other: &NavItem) -> navigation::Item {
        if !other.children.is_empty() {
            navigation::Item::Link {
                label: other.text.clone(),
                href: Some(other.url.clone()),
                external_href: None,
                http_method: None,
                title: None,
                collapsed: Some(false),
                collapsible: Some(false),
                items: Some(other.children.iter().map(|i| i.into()).collect::<Vec<_>>()),
            }
        } else {
            navigation::Item::Link {
                label: other.text.clone(),
                href: Some(other.url.clone()),
                external_href: None,
                http_method: None,
                title: None,
                collapsed: None,
                collapsible: None,
                items: None,
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::navigation::{Item, Section};
    use crate::RenderOptions;
    use pretty_assertions::assert_eq;

    #[test]
    fn parse_navigation() {
        let navigation = indoc! {"
        * [Root](/README.md)

        # An Section

        * [Installation](/Installation.md)
        * [Migrating](/Migrating.md)
        "};

        let ctx = RenderContext::new();

        let navigation = Navigation::from_markdown(navigation, &ctx).unwrap();

        assert_eq!(
            navigation,
            navigation::Navigation::new(vec![
                Section {
                    heading: None,
                    collapsed: false,
                    collapsible: false,
                    items: vec![Item::Link {
                        label: "Root".to_owned(),
                        href: Some("/README.md".to_owned()),
                        external_href: None,
                        http_method: None,
                        title: None,
                        collapsed: None,
                        collapsible: None,
                        items: None,
                    }],
                },
                Section {
                    heading: Some("An Section".to_owned()),
                    collapsed: false,
                    collapsible: false,
                    items: vec![
                        Item::Link {
                            label: "Installation".to_owned(),
                            href: Some("/Installation.md".to_owned()),
                            external_href: None,
                            http_method: None,
                            title: None,
                            collapsed: None,
                            collapsible: None,
                            items: None,
                        },
                        Item::Link {
                            label: "Migrating".to_owned(),
                            href: Some("/Migrating.md".to_owned()),
                            external_href: None,
                            http_method: None,
                            title: None,
                            collapsed: None,
                            collapsible: None,
                            items: None,
                        },
                    ],
                },
            ])
        )
    }

    #[test]
    fn parse_navigation_nested() {
        let navigation = indoc! {"
        # An Section

        * [Parent](/Parent.md)
            * [Nested](/Nested.md)
        "};

        let ctx = RenderContext::new();

        let navigation = Navigation::from_markdown(navigation, &ctx).unwrap();

        assert_eq!(
            navigation,
            navigation::Navigation::new(vec![Section {
                heading: Some("An Section".to_owned()),
                collapsible: false,
                collapsed: false,
                items: vec![Item::Link {
                    label: "Parent".to_owned(),
                    href: Some("/Parent.md".to_owned()),
                    external_href: None,
                    http_method: None,
                    title: None,
                    collapsible: Some(false),
                    collapsed: Some(false),
                    items: Some(vec![Item::Link {
                        label: "Nested".to_owned(),
                        href: Some("/Nested.md".to_owned()),
                        external_href: None,
                        http_method: None,
                        title: None,
                        collapsed: None,
                        collapsible: None,
                        items: None,
                    },]),
                },],
            },])
        )
    }

    #[test]
    fn parse_navigation_bug_section_without_items() {
        let navigation = indoc! {"
        # An Section

        # Another Section
        "};

        let ctx = RenderContext::new();

        let navigation = Navigation::from_markdown(navigation, &ctx).unwrap();

        assert_eq!(
            navigation,
            navigation::Navigation::new(vec![
                navigation::Section {
                    heading: Some("An Section".to_owned()),
                    collapsed: false,
                    collapsible: false,
                    items: vec![],
                },
                navigation::Section {
                    heading: Some("Another Section".to_owned()),
                    collapsed: false,
                    collapsible: false,
                    items: vec![],
                },
            ])
        )
    }

    #[test]
    fn parse_navigation_bug_extra_section() {
        let navigation = indoc! {"
        ## Description

        * [Behavior](Behavior.md)
        * [Diet](Diet.md)
          * [Diet](Diet.md)
            * [Diet](Diet.md)
        * [Facts](Facts.md)

        ## Status

        * [Conservation](Conservation.md)

        "};

        let ctx = RenderContext::new();

        let navigation = Navigation::from_markdown(navigation, &ctx).unwrap();

        assert_eq!(navigation.len(), 2);
    }

    #[test]
    fn set_url_prefixes_and_webbify() {
        let input = indoc! {r#"
        * [Behavior](Behavior.md)
        "#};

        let opts = RenderOptions {
            prefix_link_urls: Some("/foo".to_string()),
            webbify_internal_urls: true,
            ..Default::default()
        };
        let mut ctx = RenderContext::new();
        ctx.with_maybe_options(Some(&opts));

        let navigation = Navigation::from_markdown(input, &ctx).unwrap();

        assert_eq!(navigation[0].items[0].href(), Some("/foo/Behavior"));
    }
}
