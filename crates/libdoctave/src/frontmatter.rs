use crate::{
    markdown::error_renderer::{self, Highlight, Location},
    render_context::RenderContext,
};

fn default_as_true() -> bool {
    true
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
pub struct Frontmatter {
    #[serde(default)]
    pub hide_side_table_of_contents: bool,
    #[serde(default = "default_as_true")]
    pub toc: bool,
    #[serde(default)]
    pub hide_navigation: bool,
    #[serde(default = "default_as_true")]
    pub navigation: bool,
    #[serde(default)]
    pub page_width: PageWidth,
    pub title: Option<String>,
    #[serde(default)]
    pub meta: FrontmatterMeta,
    #[serde(default = "default_as_true")]
    pub breadcrumbs: bool,
    #[serde(default)]
    pub search: Search,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Frontmatter {
            hide_side_table_of_contents: false,
            toc: true,
            hide_navigation: false,
            navigation: true,
            title: None,
            meta: FrontmatterMeta::default(),
            breadcrumbs: true,
            page_width: PageWidth::default(),
            search: Search::default(),
        }
    }
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq, Clone)]
#[serde(rename_all = "snake_case")]
pub enum PageWidth {
    Full,
    #[default]
    Prose,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct FrontmatterMeta {
    pub description: Option<String>,
}

#[derive(Deserialize, Serialize, Debug, Default, PartialEq)]
pub struct Search {
    // Defaults to false
    #[serde(default)]
    pub hidden: bool,
}

pub fn parse(input: &str) -> std::result::Result<Frontmatter, String> {
    let pos = end_pos(input);

    let yaml = input[0..pos]
        .trim_end()
        .trim_end_matches('-')
        .trim_start()
        .trim_start_matches('-');

    serde_yaml::from_str(yaml).map_err(|e| {
        let location = if let Some(loc) = e.location() {
            Location::Point(loc.line(), loc.column())
        } else {
            Location::Point(0, 0)
        };
        let highlights = vec![Highlight {
            span: 1,
            location: location.clone(),
            msg: Some(e.to_string()),
        }];
        let ctx = RenderContext::default();
        error_renderer::render(yaml, "Invalid frontmatter", highlights, &ctx)
    })
}

/// Splits input into its frontmatter (excluding ---) and content
pub fn split(input: &str) -> (&str, &str) {
    let pos = end_pos(input);
    let yaml = input[0..pos]
        .trim_end()
        .trim_end_matches('-')
        .trim_start()
        .trim_start_matches('-');
    let content = &input[pos..];

    (yaml, content)
}

pub fn without(input: &str) -> &str {
    &input[end_pos(input)..]
}

pub fn end_pos(input: &str) -> usize {
    if let Some(after_starter_mark) = input.strip_prefix("---\n") {
        let end_mark = after_starter_mark.find("---\n");

        if let Some(e) = end_mark {
            e + 8
        } else {
            0
        }
    } else if let Some(after_starter_mark) = input.strip_prefix("---\r\n") {
        let end_mark = after_starter_mark.find("---\r\n");

        if let Some(e) = end_mark {
            e + 10
        } else {
            0
        }
    } else {
        0
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_str_eq;

    use super::*;

    #[test]
    fn basic_parse() {
        let input = indoc! {"
            ---
            title: Runbooks
            ---

            # Runbooks
        "};

        let values = parse(input).unwrap();

        let expected = Some("Runbooks".to_string());
        let actual = values.title;

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_with_objects() {
        let input = indoc! {r#"
            ---
            title: Runbooks
            meta:
                description: "foo"
            ---

            # Runbooks
        "#};

        let values = parse(input).unwrap();

        let title = values.title;
        let metadata_description = values.meta.description;

        assert_eq!(metadata_description, Some("foo".to_string()));
        assert_eq!(title, Some("Runbooks".to_string()));
    }

    #[test]
    fn missing_frontmatter() {
        let input = indoc! {"
            # Some content
        "};

        let values = parse(input).unwrap();

        assert_eq!(values, Frontmatter::default());
    }

    #[test]
    fn invalid_yaml() {
        let input = indoc! {"
            ---
            :::blarg: @@!~
            ---

            # Some content
        "};

        assert_str_eq!(
            parse(input).unwrap_err(),
            indoc! { r#"
            Invalid frontmatter

                1 │
                2 │ :::blarg: @@!~
                              ▲
                              └─ found character that cannot start any token at line 2 column 11, while scanning for the next token

            "# }
        );
    }

    #[test]
    fn never_ending_frontmatter() {
        let input = indoc! {"
            ---
            title: Runbooks

            # Runbooks
        "};

        assert_eq!(parse(input).unwrap(), Frontmatter::default());
    }

    #[test]
    fn empty_frontmatter() {
        let input = indoc! {"
            ---
            ---
            # Hi
        "};

        assert_eq!(parse(input).unwrap(), Frontmatter::default());
    }

    #[test]
    fn without_basic() {
        let input = indoc! {"
            ---
            title: Runbooks
            ---

            # Runbooks
        "};

        let without_frontmatter = without(input);

        assert_eq!(without_frontmatter, "\n# Runbooks\n");
    }

    #[test]
    fn unknown_keys_are_ignored() {
        let input = indoc! {"
            ---
            foo: bar
            ---

            # Runbooks
        "};

        assert_eq!(parse(input).unwrap(), Frontmatter::default());
    }

    #[test]
    fn windows_line_endings() {
        let input = "---\r\ntitle: Runbooks\r\n---\r\n\r\n# More content\r\n";

        let values = parse(input).unwrap();

        let expected = Some("Runbooks".to_string());
        let actual = values.title;

        assert_eq!(actual, expected);

        let without_frontmatter = without(input);

        assert_eq!(without_frontmatter, "\r\n# More content\r\n");
    }
}
