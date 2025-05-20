use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

use crate::{
    error_renderer::{self, Highlight, Location},
    page_kind::PageKind,
    render_context::RenderContext,
    Point, Position, Project,
};

pub fn parse_vale_results(json_string: &str) -> Result<ValeResults, serde_json::Error> {
    serde_json::from_str(json_string)
}

pub fn vale_results_to_errors(project: &Project, vale_results: ValeResults) -> Vec<crate::Error> {
    let mut errors = Vec::new();
    for (file, file_results) in vale_results {
        let md_content = project
            .get_page_by_fs_path(Path::new(&file))
            .and_then(|page| match page.page {
                PageKind::Markdown(ref md) => Some(md.content.clone()),
                _ => None,
            })
            .unwrap_or_default();
        for result in file_results {
            let position = Position {
                start: Point {
                    col: result.span[0] as usize,
                    row: result.line as usize,
                    byte_offset: 0,
                },
                end: Point {
                    col: result.span[1] as usize,
                    row: result.line as usize,
                    byte_offset: 0,
                },
            };

            let highlights = vec![Highlight {
                location: Location::Point(position.start.row, position.start.col),
                span: position.end.col - position.start.col + 1,
                msg: Some(result.message.clone()),
            }];

            let mut desc = error_renderer::render(
                &md_content,
                &format!("{} ({})", result.message, result.check),
                highlights,
                &RenderContext::default(),
            );

            if !result.link.is_empty() {
                desc.push_str(&formatdoc! {
                r#"

                {}, {}
                "#,
                result.check,
                result.link
                });
            }

            errors.push(crate::Error {
                code: crate::Error::VALE_ERROR,
                message: format!("{} (Vale)", result.message),
                description: desc,
                file: Some(PathBuf::from(file.clone())),
                position: Some(position),
            });
        }
    }
    errors
}

pub fn parse_vale_runtime_error(json_string: &str) -> Result<ValeRuntimeError, serde_json::Error> {
    serde_json::from_str(json_string)
}

pub fn vale_runtime_error_to_error(
    vale_runtime_error: ValeRuntimeError,
    config_path: &str,
) -> crate::Error {
    crate::Error {
        code: crate::Error::VALE_ERROR,
        message: "Vale encountered an error".to_string(),
        description: vale_runtime_error.text,
        file: Some(PathBuf::from(config_path)),
        position: None,
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ValeRuntimeError {
    pub code: String,
    pub text: String,
}

pub type ValeResults = HashMap<String, Vec<ValeResult>>;

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ValeResult {
    action: ValeAction,
    check: String,
    description: String,
    line: i32,
    link: String,
    message: String,
    severity: ValeSeverity,
    span: Vec<i32>,
    r#match: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "PascalCase")]
pub struct ValeAction {
    name: String,
    params: Option<Vec<String>>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum ValeSeverity {
    Error,
    Warning,
}

#[cfg(test)]
mod test {
    use crate::{InputContent, InputFile, NAVIGATION_FILE_NAME, SETTINGS_FILE_NAME};

    use super::*;
    use pretty_assertions::{assert_eq, assert_str_eq};

    #[test]
    fn parses() {
        let json = indoc! {
        r#"
        {
          "README.md": [
            {
              "Action": {
                "Name": "suggest",
                "Params": [
                  "spellings"
                ]
              },
              "Span": [
                1,
                5
              ],
              "Check": "Vale.Spelling",
              "Description": "",
              "Link": "",
              "Message": "Did you really mean 'asdf'?",
              "Severity": "error",
              "Match": "asdf",
              "Line": 21
            }
          ]
        }
        "#
        };

        let results = parse_vale_results(json).unwrap();
        let result_vec = results.get("README.md").unwrap();

        assert_eq!(
            *result_vec,
            vec![ValeResult {
                action: ValeAction {
                    name: "suggest".to_string(),
                    params: Some(vec!["spellings".to_string()]),
                },
                check: "Vale.Spelling".to_string(),
                description: "".to_string(),
                line: 21,
                link: "".to_string(),
                message: "Did you really mean 'asdf'?".to_string(),
                severity: ValeSeverity::Error,
                span: vec![1, 5],
                r#match: "asdf".to_string(),
            }]
        );
    }

    #[test]
    fn results_to_errors() {
        let files = vec![
            InputFile {
                path: PathBuf::from(NAVIGATION_FILE_NAME),
                content: InputContent::Text("".to_owned()),
            },
            InputFile {
                path: PathBuf::from(SETTINGS_FILE_NAME),
                content: InputContent::Text(
                    indoc! { r#"
                ---
                title: An Project
                version: 2
                "# }
                    .to_owned(),
                ),
            },
            InputFile {
                path: PathBuf::from("README.md"),
                content: InputContent::Text(
                    indoc! {r#"
                    # Hi

                    This is some asdf
                    "#}
                    .to_owned(),
                ),
            },
        ];

        let project = Project::from_file_list(files).unwrap();

        let results = vec![
            ValeResult {
                action: ValeAction {
                    name: "suggest".to_string(),
                    params: Some(vec!["spellings".to_string()]),
                },
                check: "Vale.Spelling".to_string(),
                description: "".to_string(),
                line: 3,
                link: "".to_string(),
                message: "Did you really mean 'asdf'?".to_string(),
                severity: ValeSeverity::Warning,
                span: vec![14, 17],
                r#match: "asdf".to_string(),
            },
            ValeResult {
                action: ValeAction {
                    name: "suggest".to_string(),
                    params: Some(vec!["spellings".to_string()]),
                },
                check: "Vale.Spelling".to_string(),
                description: "".to_string(),
                line: 21,
                link: "".to_string(),
                message: "Did you really mean 'asdf'?".to_string(),
                severity: ValeSeverity::Error,
                span: vec![1, 5],
                r#match: "asdf".to_string(),
            },
        ];

        let vale_results = HashMap::from_iter(vec![("README.md".to_string(), results.clone())]);
        let errors = vale_results_to_errors(&project, vale_results);
        assert_eq!(errors.len(), 2);
        assert_eq!(errors[0].code, crate::Error::VALE_ERROR);
        assert_str_eq!(
            errors[0].description,
            indoc! {
            r#"
            Did you really mean 'asdf'? (Vale.Spelling)

                2 │
                3 │ This is some asdf
                                 ▲▲▲▲
                                 └─ Did you really mean 'asdf'?

            "#
            }
        );
    }
}
