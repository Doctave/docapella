use indoc::indoc;
use libdoctave::{InputContent, InputFile, Project, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME};
use std::path::PathBuf;

static GITHUB_SPEC: &str = include_str!("../examples/open_api_specs/github.yaml");

/// Render each section of the Github OpenAPI spec
///
/// NOTE: This is currently slow...
///
fn main() {
    // Check we can parse it
    let file_list = vec![
        InputFile {
            path: PathBuf::from("README.md"),
            content: InputContent::Text(String::from("")),
        },
        InputFile {
            path: PathBuf::from("openapi.json"),
            content: InputContent::Text(GITHUB_SPEC.to_owned()),
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
          open_api:
            - spec_file: openapi.json
        "#})),
        },
    ];

    let project = Project::from_file_list(file_list).unwrap();

    // And render it
    for page in project.pages() {
        println!("Page {:?}", page.title());
        page.ast(None).unwrap();
    }
}
