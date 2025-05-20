#![allow(dead_code)]

use indoc::{formatdoc, indoc};
use libdoctave::{
    Error, InputContent, InputFile, Project, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME,
};
use std::path::PathBuf;

static MDFILE: &str = include_str!("../examples/all_markdown_features/README.md");

pub struct ProjectBuilder {
    pub inputs: Vec<InputFile>,
}

impl Default for ProjectBuilder {
    fn default() -> Self {
        ProjectBuilder {
            inputs: vec![
                InputFile {
                    path: PathBuf::from("README.md"),
                    content: InputContent::Text(String::from("")),
                },
                InputFile {
                    path: PathBuf::from(SETTINGS_FILE_NAME),
                    content: InputContent::Text(
                        indoc! {"
                            ---
                            title: An Project
                            "}
                        .to_owned(),
                    ),
                },
                InputFile {
                    path: PathBuf::from(NAVIGATION_FILE_NAME),
                    content: InputContent::Text("---\n".to_owned()),
                },
            ],
        }
    }
}

impl ProjectBuilder {
    pub fn n_project_files(n: usize) -> Vec<InputFile> {
        let arr = vec![0; n];

        let mut navigation = indoc! {r#"
        ---
        "#}
        .to_owned();

        let mut nav_part = String::new();

        let mut files: Vec<_> = arr
            .iter()
            .enumerate()
            .map(|(i, _)| {
                if i % 100 == 0 {
                    navigation.push_str(&nav_part);

                    nav_part = String::new();

                    nav_part.push_str(&formatdoc! { r#"
                - heading: SECTION_{}
                  items:
                "#, i})
                }

                nav_part.push_str(&format!(
                    "  - label: DOC_{}\n    href: /docs/DOC_{}\n",
                    i, i
                ));

                let mut content = MDFILE.to_string();

                content.push_str("\n\n{% include \"_partials/todo.md\" %}");

                InputFile {
                    path: PathBuf::from(format!("docs/DOC_{}.md", i)),
                    content: InputContent::Text(MDFILE.to_string()),
                }
            })
            .collect();

        let todo = indoc! {"
        _Edit me at `_partials/todo.md`._

        * [ ] Create a new page
        * [ ] Use a partial template
        * [ ] Read the [official guide](http://docs.doctave.com)
        "};

        let mut setup_files = vec![
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(String::from("")),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! {"
                        ---
                        title: An Project
                        "}
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("_partials/todo.md"),
                content: InputContent::Text(todo.to_owned()),
            },
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text(navigation),
            },
        ];

        files.append(&mut setup_files);

        files
    }

    pub fn with_openapi(mut self, spec: String) -> Self {
        self.with_file("openapi.json", spec);
        self.append_to_file(
            libdoctave::SETTINGS_FILE_NAME,
            indoc! { r##"
        open_api:
          - spec_file: openapi.json
            uri_prefix: /api
        "##},
        )
        .append_to_file(
            libdoctave::NAVIGATION_FILE_NAME,
            indoc! {r#"
            - heading: API
              items:
                - open_api_spec: openapi.json
            "#},
        )
    }

    pub fn with_file<P: Into<PathBuf>, S: Into<String>>(&mut self, path: P, content: S) {
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

    pub fn build(self) -> std::result::Result<Project, Vec<Error>> {
        Project::from_file_list(self.inputs)
    }

    fn append_to_file<P: Into<PathBuf>>(mut self, path: P, text: &str) -> Self {
        let path = path.into();

        if let Some(pos) = self.inputs.iter_mut().position(|i| i.path == path) {
            let file = self.inputs.remove(pos);
            let new_content = match file.content {
                InputContent::Binary(_) => unimplemented!(),
                InputContent::Text(mut content) => {
                    content.push_str(text);
                    content
                }
            };
            self.inputs.push(InputFile {
                path: file.path,
                content: InputContent::Text(new_content),
            })
        }
        self
    }
}
